use super::constants::*;
use super::debug;
use super::error::Error;
use super::types::Configs;
use super::util::{blake2b_256, find_cells_by_script, script_literal_to_script};
use ckb_std::{ckb_constants::Source, high_level};
use core::convert::{TryFrom, TryInto};
use das_types::{
    constants::{ConfigID, DataType, WITNESS_HEADER},
    packed::*,
    prelude::*,
};
use std::prelude::v1::*;

#[derive(Debug)]
pub struct WitnessesParser {
    pub witnesses: Vec<Vec<u8>>,
    action: Option<ActionData>,
    configs: Option<Configs>,
    // The Bytes is wrapped DataEntity.entity.
    dep: Vec<(u32, u32, u32, Vec<u8>, Bytes)>,
    old: Vec<(u32, u32, u32, Vec<u8>, Bytes)>,
    new: Vec<(u32, u32, u32, Vec<u8>, Bytes)>,
}

impl WitnessesParser {
    pub fn new(witnesses: Vec<Vec<u8>>) -> Result<Self, Error> {
        if witnesses.len() <= 0 {
            return Err(Error::WitnessEmpty);
        }

        Ok(WitnessesParser {
            witnesses,
            action: None,
            configs: None,
            dep: Vec::new(),
            old: Vec::new(),
            new: Vec::new(),
        })
    }

    pub fn action(&self) -> (&[u8], &[u8]) {
        let action_data = self.action.as_ref().unwrap().as_reader();

        (
            action_data.action().raw_data(),
            action_data.params().raw_data(),
        )
    }

    pub fn configs(&self) -> &Configs {
        self.configs.as_ref().unwrap()
    }

    pub fn parse_only_action(&mut self) -> Result<(), Error> {
        debug!("Parsing action witness only ...");

        let witness = self.witnesses.get(0).ok_or(Error::WitnessEmpty)?;
        Self::verify_das_header(witness.as_slice())?;
        let type_ = Self::parse_data_type(&witness)?;
        if type_ != DataType::ActionData as u32 {
            return Err(Error::WitnessActionIsNotTheFirst);
        }

        self.action = Some(Self::parse_action_data(&witness)?);

        Ok(())
    }

    pub fn parse_only_config(&mut self, config_ids: &[ConfigID]) -> Result<(), Error> {
        debug!("Parsing config witnesses only ...");

        debug!("  Load ConfigCells in cell_deps ...");

        let config_cell_type = script_literal_to_script(CONFIG_CELL_TYPE);
        let mut config_data_types = Vec::new();
        let mut config_entity_hashes = Vec::new();
        for config_id in config_ids {
            let args = Bytes::from((config_id.to_owned() as u32).to_le_bytes().to_vec());
            let type_script = config_cell_type
                .clone()
                .as_builder()
                .args(args.into())
                .build();
            // There must be one ConfigCell in the cell_deps, no more and no less.
            let ret = find_cells_by_script(ScriptType::Type, &type_script, Source::CellDep)?;
            if ret.len() != 1 {
                return Err(Error::ConfigCellIsRequired);
            }
            let expected_cell_index = ret[0];

            let data = high_level::load_cell_data(expected_cell_index, Source::CellDep)
                .map_err(|e| Error::from(e))?;
            let expected_entity_hash = match data.get(..32) {
                Some(bytes) => bytes.to_owned(),
                _ => return Err(Error::InvalidCellData),
            };

            debug!(
                "    Load ConfigCell ID: {:?} Hash: {:?}",
                config_id, expected_entity_hash
            );

            // Store entity hash for later verification.
            config_entity_hashes.push(expected_entity_hash);

            // Store data type for loading data on demand.
            match config_id {
                ConfigID::ConfigCellMain => config_data_types.push(DataType::ConfigCellMain),
                ConfigID::ConfigCellRegister => {
                    config_data_types.push(DataType::ConfigCellRegister)
                }
                ConfigID::ConfigCellBloomFilter => {
                    config_data_types.push(DataType::ConfigCellBloomFilter)
                }
                ConfigID::ConfigCellMarket => config_data_types.push(DataType::ConfigCellMarket),
            }
        }

        debug!("  Load witnesses of the ConfigCells ...");

        let mut configs = Configs::new();
        for (_i, witness) in self.witnesses.iter().enumerate().skip(1) {
            Self::verify_das_header(&witness)?;

            // Just handle required config witness.
            let data_type = DataType::try_from(Self::parse_data_type(&witness)?).unwrap();
            if !config_data_types.contains(&data_type) {
                continue;
            }

            let entity = Self::verify_hash_and_get_entity(witness, &mut config_entity_hashes)?;
            debug!("    Found matched ConfigCell witness at: witnesses[{}]", _i);
            match data_type {
                DataType::ConfigCellMain => {
                    configs.main = Some(
                        ConfigCellMain::from_slice(entity)
                            .map_err(|_| Error::ConfigCellWitnessDecodingError)?,
                    );
                }
                DataType::ConfigCellRegister => {
                    configs.register = Some(
                        ConfigCellRegister::from_slice(entity)
                            .map_err(|_| Error::ConfigCellWitnessDecodingError)?,
                    );
                }
                DataType::ConfigCellBloomFilter => {
                    configs.bloom_filter = Some(entity.get(4..).unwrap().to_vec());
                }
                DataType::ConfigCellMarket => {
                    configs.market = Some(
                        ConfigCellMarket::from_slice(entity)
                            .map_err(|_| Error::ConfigCellWitnessDecodingError)?,
                    );
                }
                _ => return Err(Error::ConfigIDIsUndefined),
            }
        }

        // Check if there is any hash is not used, which means some config is missing.
        if config_entity_hashes.len() > 0 {
            debug!("Can not find some ConfigCells' witnesses.");
            return Err(Error::ConfigIsPartialMissing);
        }

        self.configs = Some(configs);

        Ok(())
    }

    fn verify_hash_and_get_entity<'a>(
        witness: &'a Vec<u8>,
        config_entity_hashes: &mut Vec<Vec<u8>>,
    ) -> Result<&'a [u8], Error> {
        let raw = witness
            .get(7..11)
            .ok_or(Error::ConfigCellWitnessDecodingError)?;
        let length = u32::from_le_bytes(raw.try_into().unwrap()) as usize;

        let entity = witness
            .get(7..(7 + length))
            .ok_or(Error::ConfigCellWitnessDecodingError)?;
        let entity_hash = blake2b_256(entity).to_vec();
        let ret = config_entity_hashes
            .iter()
            .enumerate()
            .find(|&(_, item)| item.as_slice() == entity_hash.as_slice());

        if let Some((i, _)) = ret {
            config_entity_hashes.remove(i);
        } else {
            // ⚠️ Do not print the whole entity, otherwise memory may be not enough.
            debug!(
                "Corrupted witness found! hash: {:?} entity: {:?}",
                entity_hash,
                entity.get(..10)
            );
            return Err(Error::ConfigCellWitnessIsCorrupted);
        }

        Ok(entity)
    }

    pub fn parse_all_data(&mut self) -> Result<(), Error> {
        debug!("Parsing witnesses of all other cells ...");

        for witness in self.witnesses.iter().skip(1) {
            Self::verify_das_header(witness)?;

            let data_type = Self::parse_data_type(witness)?;

            // Skip all config witnesses.
            if self.is_config_data_type(data_type) {
                continue;
            }

            let data = Self::parse_data(witness)?;
            if let Some(entity) = data.dep().to_opt() {
                self.dep.push(Self::parse_entity(entity, data_type)?)
            }
            if let Some(entity) = data.old().to_opt() {
                self.old.push(Self::parse_entity(entity, data_type)?)
            }
            if let Some(entity) = data.new().to_opt() {
                self.new.push(Self::parse_entity(entity, data_type)?)
            }
        }

        Ok(())
    }

    fn verify_das_header(witness: &[u8]) -> Result<(), Error> {
        if let Some(raw) = witness.get(..3) {
            if raw != &WITNESS_HEADER {
                return Err(Error::WitnessDasHeaderDecodingError);
            }
        } else {
            return Err(Error::WitnessDasHeaderDecodingError);
        };

        Ok(())
    }

    fn parse_data_type(witness: &[u8]) -> Result<u32, Error> {
        if let Some(raw) = witness.get(3..7) {
            let type_in_int = match Uint32Reader::verify(raw, false) {
                Ok(()) => Uint32::new_unchecked(raw.to_vec().into()),
                Err(_) => return Err(Error::WitnessTypeDecodingError),
            };
            Ok(u32::from(type_in_int))
        } else {
            Err(Error::WitnessTypeDecodingError)
        }
    }

    fn parse_action_data(witness: &[u8]) -> Result<ActionData, Error> {
        if let Some(raw) = witness.get(7..) {
            let action_data = match ActionData::from_slice(raw) {
                Ok(action_data) => action_data,
                Err(_) => return Err(Error::WitnessActionDecodingError),
            };
            Ok(action_data)
        } else {
            Err(Error::WitnessActionDecodingError)
        }
    }

    fn parse_data(witness: &[u8]) -> Result<Data, Error> {
        if let Some(raw) = witness.get(7..11) {
            // Because of the redundancy of the witness, appropriate trimming is performed here.
            let length = u32::from_le_bytes(raw.try_into().unwrap()) as usize;
            if let Some(raw) = witness.get(7..(7 + length)) {
                let data = match Data::from_slice(raw) {
                    Ok(data) => data,
                    Err(_e) => {
                        debug!("WitnessDataDecodingError: {:?}", _e);
                        return Err(Error::WitnessDataDecodingError);
                    }
                };
                Ok(data)
            } else {
                Err(Error::WitnessDataReadDataBodyFailed)
            }
        } else {
            Err(Error::WitnessDataParseLengthHeaderFailed)
        }
    }

    fn parse_entity(
        entity: DataEntity,
        entity_type: u32,
    ) -> Result<(u32, u32, u32, Vec<u8>, Bytes), Error> {
        let index = u32::from(entity.index());
        let version = u32::from(entity.version());
        let data = entity.entity();

        let entity_data = data
            .as_slice()
            .get(4..)
            .ok_or(Error::WitnessEntityMissing)?;
        let hash = blake2b_256(entity_data).to_vec();

        // debug!(
        //     "entity: index = {} hash = {:?} entity = {:?}",
        //     index, hash, data
        // );

        Ok((
            index,
            version,
            entity_type,
            hash,
            Bytes::new_unchecked(data.as_bytes()),
        ))
    }

    pub fn verify_and_get(
        &self,
        index: usize,
        source: Source,
    ) -> Result<(u32, u32, &Bytes), Error> {
        let data = high_level::load_cell_data(index, source).map_err(|e| Error::from(e))?;
        let hash = match data.get(..32) {
            Some(bytes) => bytes.to_vec(),
            _ => return Err(Error::InvalidCellData),
        };

        Ok(self.get(index as u32, &hash, source)?)
    }

    pub fn get(
        &self,
        index: u32,
        hash: &[u8],
        source: Source,
    ) -> Result<(u32, u32, &Bytes), Error> {
        let group = match source {
            Source::Input => &self.old,
            Source::Output => &self.new,
            Source::CellDep => &self.dep,
            _ => {
                return Err(Error::HardCodedError);
            }
        };

        let entity;
        let version;
        let entity_type;
        if let Some((_, _version, _entity_type, _hash, _entity)) =
            group.iter().find(|&(i, _, _, _h, _)| i == &index)
        {
            if hash == _hash.as_slice() {
                version = _version.to_owned();
                entity_type = _entity_type.to_owned();
                entity = _entity;
            } else {
                debug!(
                    "Witness hash verify failed: {:?}[{}] {:?}",
                    source, index, hash
                );
                return Err(Error::WitnessDataIsCorrupted);
            }
        } else {
            debug!(
                "Can not find witness at: {:?}[{}] {:?}",
                source, index, hash
            );
            return Err(Error::WitnessDataMissing);
        }

        // This is DataEntity.entity wrapped in Bytes.
        Ok((version, entity_type, entity))
    }

    fn is_config_data_type(&self, data_type: u32) -> bool {
        let config_data_types = [
            DataType::ConfigCellMain as u32,
            DataType::ConfigCellRegister as u32,
            DataType::ConfigCellBloomFilter as u32,
            DataType::ConfigCellMarket as u32,
        ];

        config_data_types.contains(&data_type)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::is_reader_eq;
    use das_types::util::is_entity_eq;

    fn restore_bytes_from_hex(input: &str) -> Vec<u8> {
        let trimed_input = input.trim_start_matches("0x");
        hex::decode(trimed_input).unwrap()
    }

    fn before_each() -> WitnessesParser {
        let witnesses = vec![
            restore_bytes_from_hex("0x646173000000001a0000000c0000001600000006000000636f6e66696700000000"),
            restore_bytes_from_hex("0x6461730a00000060010000100000001400000018000000008d27002c0100004801000028000000480000006800000088000000a8000000c8000000e80000000801000028010000cac501b0a5826bffa485ccac13c2195fcdf3aa86b113203f620ddd34d3decd70431a3af2d4bbcd69ab732d37be794ac0ab172c151545dfdbae1f578a7083bc84071ee1a005b5bc1a619aed290c39bbb613ac93991eabab8418d6b0a9bdd220eb15f69a14cfafac4e21516e7076e135492c4b20fe4fb5af9e1942577a46985a133d216e5bfb54b9e2ec0f0fbb1cdf23703f550a7ec7c35264742fce69308482e1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000cf2f19e19c13d4ccfeae96634f6be6cdb2e4cd68f810ce3b865ee34030374524"),
            restore_bytes_from_hex("0x6461730a0000003b0400003000000034000000380000003c000000e8020000140400001504000016040000170400001b0400001f0400003c00000080510100e8030000ac020000100000004c000000bf0000003c00000010000000140000001500000000000000012700000010000000180000002000000004000000f09f988204000000f09f918d03000000e29ca87300000010000000140000001500000001000000015e0000002c00000031000000360000003b00000040000000450000004a0000004f00000054000000590000000100000030010000003101000000320100000033010000003401000000350100000036010000003701000000380100000039ed0100001000000014000000150000000200000000d8010000d4000000d9000000de000000e3000000e8000000ed000000f2000000f7000000fc00000001010000060100000b01000010010000150100001a0100001f01000024010000290100002e01000033010000380100003d01000042010000470100004c01000051010000560100005b01000060010000650100006a0100006f01000074010000790100007e01000083010000880100008d01000092010000970100009c010000a1010000a6010000ab010000b0010000b5010000ba010000bf010000c4010000c9010000ce010000d3010000010000006101000000620100000063010000006401000000650100000066010000006701000000680100000069010000006a010000006b010000006c010000006d010000006e010000006f0100000070010000007101000000720100000073010000007401000000750100000076010000007701000000780100000079010000007a010000004101000000420100000043010000004401000000450100000046010000004701000000480100000049010000004a010000004b010000004c010000004d010000004e010000004f0100000050010000005101000000520100000053010000005401000000550100000056010000005701000000580100000059010000005a2c01000024000000450000006600000087000000a8000000c9000000ea0000000b01000021000000100000001100000019000000044054890000000000a0bb0d00000000002100000010000000110000001900000008404b4c000000000020a10700000000002100000010000000110000001900000002c0d8a70000000000e0c8100000000000210000001000000011000000190000000500127a000000000000350c00000000002100000010000000110000001900000006c0cf6a000000000060ae0a00000000002100000010000000110000001900000007808d5b0000000000c0270900000000002100000010000000110000001900000001001bb70000000000804f1200000000002100000010000000110000001900000003809698000000000040420f000000000004020632000000320000001c000000100000001400000018000000e8030000e8030000401f0000"),
            restore_bytes_from_hex("0x646173090000000000000000000000000000000000"),
            restore_bytes_from_hex("0x6461730a000000540000000c000000300000002400000014000000180000001c00000020000000000000000000000080510100e80300002400000014000000180000001c00000020000000008d2700008d270080510100e8030000"),
        ];

        WitnessesParser::new(witnesses).unwrap()
    }

    #[test]
    fn test_parse_only_action() {
        let mut parser = before_each();
        parser.parse_only_action().unwrap();
        let (action, params) = parser.action();

        assert!(action == b"config", "Action should be \"config\".");
        assert!(params == &[], "Params should be empty.");
    }

    // #[test]
    fn test_parse_all_data() {
        let mut parser = before_each();
        parser.parse_all_data().unwrap();

        let hash = restore_bytes_from_hex(
            "0x000045a843802e1c0bb8f1e382ee23be1434c36693eac143f61bbaf04dc90000",
        );
        let (version, data_type, entity) = parser.get(0, hash.as_slice(), Source::Output).unwrap();

        assert_eq!(1, version);
    }

    #[test]
    #[should_panic]
    fn test_verify_das_header() {
        let witnesses = vec![
            restore_bytes_from_hex("0x21000000646173060000001a0000000c0000001600000006000000636f6e66696700000000"),
            restore_bytes_from_hex("0x00000000"),
            restore_bytes_from_hex("0x0501000064617301000000fe0000001000000010000000fe000000ee0000001000000014000000180000000000000001000000d2000000d2000000380000003c0000003d0000003e00000042000000460000004a0000004e0000005200000056000000c6000000ca000000ce00000000000000040232000000320000003c00000080510100e803000004000000700000001000000030000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0100008051010080510100"),
        ];

        let mut parser = WitnessesParser::new(witnesses).unwrap();
        parser.parse_all_data().unwrap();
    }

    // #[test]
    // #[should_panic]
    fn fn_get_should_check_hash_when_getting_data() {
        let parser = before_each();

        let hash = restore_bytes_from_hex(
            "0x000045a843802e1c0bb8f1e382ee23be1434c36693eac143f61bbaf04dc90000",
        );
        parser.get(0, hash.as_slice(), Source::Output).unwrap();
    }

    // #[test]
    fn fn_get_should_return_correctly() {
        let parser = before_each();

        let hash = restore_bytes_from_hex(
            "0x04de45a843802e1c0bb8f1e382ee23be1434c36693eac143f61bbaf04dc901cb",
        );
        let (_, _, entity) = parser.get(0, &hash, Source::Output).unwrap();

        let entity_data = entity.as_slice().get(4..).unwrap();
        let result = blake2b_256(entity_data).to_vec();

        assert!(
            hash == result,
            "The hash of the returned data should match."
        )
    }
}
