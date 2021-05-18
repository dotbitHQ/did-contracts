use super::constants::*;
use super::error::Error;
use super::types::Configs;
use super::util;
use super::{assert, debug};
use ckb_std::ckb_constants::Source;
use core::convert::{TryFrom, TryInto};
use das_types::{
    constants::{DataType, WITNESS_HEADER},
    packed::*,
    prelude::*,
};
use std::prelude::v1::*;

#[derive(Debug)]
pub struct WitnessesParser {
    pub witnesses: Vec<Vec<u8>>,
    pub configs: Configs,
    // The Bytes is wrapped DataEntity.entity.
    dep: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
    old: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
    new: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
}

impl WitnessesParser {
    pub fn new(witnesses: Vec<Vec<u8>>) -> Result<Self, Error> {
        if witnesses.len() <= 0 {
            return Err(Error::WitnessEmpty);
        }

        Ok(WitnessesParser {
            witnesses,
            configs: Configs::new(),
            dep: Vec::new(),
            old: Vec::new(),
            new: Vec::new(),
        })
    }

    pub fn parse_only_config(&mut self, config_types: &[DataType]) -> Result<(), Error> {
        debug!("Parsing config witnesses only ...");

        debug!("  Load ConfigCells in cell_deps ...");

        let config_cell_type = util::script_literal_to_script(CONFIG_CELL_TYPE);
        let mut config_data_types = Vec::new();
        let mut config_entity_hashes = Vec::new();
        for config_type in config_types {
            let args = Bytes::from((config_type.to_owned() as u32).to_le_bytes().to_vec());
            let type_script = config_cell_type
                .clone()
                .as_builder()
                .args(args.into())
                .build();
            // There must be one ConfigCell in the cell_deps, no more and no less.
            let ret = util::find_cells_by_script(ScriptType::Type, &type_script, Source::CellDep)?;
            assert!(
                ret.len() == 1,
                Error::ConfigCellIsRequired,
                "  Can not find {:?} in cell_deps. (find_condition: {})",
                config_type,
                type_script
            );

            let expected_cell_index = ret[0];
            let data = util::load_cell_data(expected_cell_index, Source::CellDep)?;
            let expected_entity_hash = match data.get(..32) {
                Some(bytes) => bytes.to_owned(),
                _ => return Err(Error::InvalidCellData),
            };

            // debug!(
            //     "    Load ConfigCell with DataType: {:?} Witness Hash: {:?}",
            //     config_type, expected_entity_hash
            // );

            // Store entity hash for later verification.
            config_entity_hashes.push(expected_entity_hash);

            // Store data type for loading data on demand.
            config_data_types.push(config_type.to_owned())
        }

        debug!("  Load witnesses of the ConfigCells ...");

        let mut configs = Configs::new();

        macro_rules! assign_config_witness {
            ( $property:expr, $witness_type:ty, $entity:expr ) => {
                $property = Some(
                    <$witness_type>::from_slice($entity)
                        .map_err(|_| Error::ConfigCellWitnessDecodingError)?,
                );
            };
        }

        macro_rules! assign_config_reserved_account_witness {
            ( $index:expr, $entity:expr ) => {
                if configs.reserved_account.is_some() {
                    configs.reserved_account.as_mut().map(|account_lists| {
                        account_lists[$index] = $entity.get(4..).unwrap().to_vec()
                    });
                } else {
                    let mut account_lists = vec![Vec::new(); 8];
                    account_lists[$index] = $entity.get(4..).unwrap().to_vec();
                    configs.reserved_account = Some(account_lists)
                }
            };
        }

        for (_i, witness) in self.witnesses.iter().enumerate() {
            Self::verify_das_header(&witness)?;

            // Just handle required config witness.
            let data_type = Self::parse_data_type(&witness)?;
            if !config_data_types.contains(&data_type) {
                continue;
            }

            let entity = Self::verify_hash_and_get_entity(_i, witness, &mut config_entity_hashes)?;
            debug!(
                "    Found matched ConfigCell witness at: witnesses[{}] data_type: {:?}",
                _i, data_type
            );
            match data_type {
                DataType::ConfigCellAccount => {
                    assign_config_witness!(configs.account, ConfigCellAccount, entity)
                }
                DataType::ConfigCellApply => {
                    assign_config_witness!(configs.apply, ConfigCellApply, entity)
                }
                DataType::ConfigCellCharSet => {
                    assign_config_witness!(configs.char_set, ConfigCellCharSet, entity)
                }
                DataType::ConfigCellIncome => {
                    assign_config_witness!(configs.income, ConfigCellIncome, entity)
                }
                DataType::ConfigCellMain => {
                    assign_config_witness!(configs.main, ConfigCellMain, entity)
                }
                DataType::ConfigCellPrice => {
                    assign_config_witness!(configs.price, ConfigCellPrice, entity)
                }
                DataType::ConfigCellProposal => {
                    assign_config_witness!(configs.proposal, ConfigCellProposal, entity)
                }
                DataType::ConfigCellProfitRate => {
                    assign_config_witness!(configs.profit_rate, ConfigCellProfitRate, entity)
                }
                DataType::ConfigCellRecordKeyNamespace => {
                    configs.record_key_namespace = Some(entity.get(4..).unwrap().to_vec());
                }
                DataType::ConfigCellPreservedAccount00 => {
                    assign_config_reserved_account_witness!(0, entity)
                }
                _ => return Err(Error::ConfigTypeIsUndefined),
            }
        }

        // Check if there is any hash is not used, which means some config is missing.
        if config_entity_hashes.len() > 0 {
            debug!("Can not find some ConfigCells' witnesses.");
            return Err(Error::ConfigIsPartialMissing);
        }

        self.configs = configs;

        Ok(())
    }

    fn verify_hash_and_get_entity<'a>(
        _i: usize,
        witness: &'a Vec<u8>,
        config_entity_hashes: &mut Vec<Vec<u8>>,
    ) -> Result<&'a [u8], Error> {
        // debug!("Calculate and verify hash of witness[{}]", _i);

        let entity = witness
            .get(7..)
            .ok_or(Error::ConfigCellWitnessDecodingError)?;
        let entity_hash = util::blake2b_256(entity).to_vec();
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
                util::hex_string(entity_hash.as_slice()),
                entity.get(..10).map(|item| util::hex_string(item) + "...")
            );
            return Err(Error::ConfigCellWitnessIsCorrupted);
        }

        Ok(entity)
    }

    pub fn parse_all_data(&mut self) -> Result<(), Error> {
        debug!("Parsing witnesses of all other cells ...");

        for (_i, witness) in self.witnesses.iter().enumerate() {
            Self::verify_das_header(witness)?;

            let data_type = Self::parse_data_type(witness)?;

            // Skip all config witnesses.
            if self.is_config_data_type(data_type) {
                continue;
            }

            // debug!("Parse witnesses[{}] in type: {:?}", _i, data_type);

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

    fn parse_data_type(witness: &[u8]) -> Result<DataType, Error> {
        if let Some(raw) = witness.get(3..7) {
            let data_type = DataType::try_from(u32::from_le_bytes(raw.try_into().unwrap()))
                .map_err(|_| Error::WitnessTypeDecodingError)?;
            Ok(data_type)
        } else {
            Err(Error::WitnessTypeDecodingError)
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
        data_entity: DataEntity,
        data_type: DataType,
    ) -> Result<(u32, u32, DataType, Vec<u8>, Bytes), Error> {
        let index = u32::from(data_entity.index());
        let version = u32::from(data_entity.version());
        let entity = data_entity.entity();

        let unwraped_entity = entity.as_reader().raw_data();
        let hash = util::blake2b_256(unwraped_entity).to_vec();

        // debug!(
        //     "entity: index = {} hash = {:?} entity = {:?}",
        //     index, hash, unwraped_entity
        // );

        Ok((index, version, data_type, hash, entity))
    }

    pub fn verify_and_get(
        &self,
        index: usize,
        source: Source,
    ) -> Result<(u32, DataType, &Bytes), Error> {
        let data = util::load_cell_data(index, source)?;
        let hash = match data.get(..32) {
            Some(bytes) => bytes.to_vec(),
            _ => return Err(Error::InvalidCellData),
        };

        let group = match source {
            Source::Input => &self.old,
            Source::Output => &self.new,
            Source::CellDep => &self.dep,
            _ => {
                return Err(Error::HardCodedError);
            }
        };

        let version;
        let data_type;
        let entity;
        if let Some((_, _version, _entity_type, _hash, _entity)) =
            group.iter().find(|&(i, _, _, _h, _)| *i as usize == index)
        {
            if hash == _hash.as_slice() {
                version = _version.to_owned();
                data_type = _entity_type.to_owned();
                entity = _entity;
            } else {
                // This error means the there is no hash(witness.data.dep/old/new.entity) matches the leading 32 bytes of the cell.
                debug!(
                    "  {:?}[{}] Witness hash verify failed: data_type: {:?}, hash_in_cell_data: 0x{} calculated_hash: 0x{} entity: 0x{}",
                    source,
                    index,
                    _entity_type,
                    util::hex_string(hash.as_slice()),
                    util::hex_string(_hash.as_slice()),
                    util::hex_string(_entity.as_reader().raw_data())
                );
                return Err(Error::WitnessDataHashMissMatch);
            }
        } else {
            // This error means the there is no witness.data.dep/old/new.index matches the index of the cell.
            debug!(
                "Can not find witness at: {:?}[{}] 0x{}",
                source,
                index,
                util::hex_string(hash.as_slice())
            );
            return Err(Error::WitnessDataIndexMissMatch);
        }

        Ok((version, data_type, entity))
    }

    fn is_config_data_type(&self, data_type: DataType) -> bool {
        let config_data_types = [
            DataType::ConfigCellAccount,
            DataType::ConfigCellApply,
            DataType::ConfigCellCharSet,
            DataType::ConfigCellIncome,
            DataType::ConfigCellMain,
            DataType::ConfigCellPrice,
            DataType::ConfigCellProposal,
            DataType::ConfigCellProfitRate,
            DataType::ConfigCellRecordKeyNamespace,
            DataType::ConfigCellPreservedAccount00,
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
}
