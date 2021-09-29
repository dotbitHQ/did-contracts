use super::{
    assert,
    constants::*,
    debug,
    error::Error,
    types::{CharSet, Configs},
    util, warn,
};
use ckb_std::{ckb_constants::Source, error::SysError, syscalls};
use core::convert::{TryFrom, TryInto};
use das_map::map;
use das_types::{
    constants::{DataType, CHAR_SET_LENGTH, WITNESS_HEADER},
    packed::*,
    prelude::*,
    util as das_types_util,
};
use std::prelude::v1::*;

#[derive(Debug)]
pub struct WitnessesParser {
    pub witnesses: Vec<(usize, DataType)>,
    pub configs: Configs,
    // The Bytes is wrapped DataEntity.entity.
    dep: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
    old: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
    new: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
}

// entity FORMAT 1: 'das'(3) + DATA_TYPE(4) + molecule
// entity FORMAT 2: 'das'(3) + DATA_TYPE(4) + binary_data(molecule like data: LENGTH(4) + ENTITY)
const DAS_BYTES_3: usize = 3;
const LENGTH_BYTES_4: usize = 4;
const HEADER_BYTES_7: usize = 7;

impl WitnessesParser {
    pub fn new() -> Result<Self, Error> {
        let mut witnesses = Vec::new();
        let mut i = 0;
        let mut das_witnesses_started = false;
        loop {
            let mut buf = [0u8; HEADER_BYTES_7];
            let ret = syscalls::load_witness(&mut buf, 0, i, Source::Input);

            match ret {
                // Data which length is too short to be DAS witnesses, so ignore it.
                Ok(_) => i += 1,
                Err(SysError::LengthNotEnough(_)) => {
                    if let Some(raw) = buf.get(..DAS_BYTES_3) {
                        if raw != &WITNESS_HEADER {
                            assert!(
                                !das_witnesses_started,
                                Error::WitnessStructureError,
                                "The witnesses of DAS must at the end of witnesses field and next to each other."
                            );

                            i += 1;
                            continue;
                        }
                    }

                    let data_type_in_int =
                        u32::from_le_bytes(buf.get(DAS_BYTES_3..HEADER_BYTES_7).unwrap().try_into().unwrap());
                    match DataType::try_from(data_type_in_int) {
                        Ok(data_type) => {
                            if !das_witnesses_started {
                                assert!(
                                    data_type == DataType::ActionData,
                                    Error::WitnessStructureError,
                                    "The first DAS witness must be the type of DataType::ActionData ."
                                );
                                das_witnesses_started = true
                            }

                            witnesses.push((i, data_type));
                        }
                        Err(_) => {
                            // Ignore unknown DataTypes which will make adding new DataType much easier and no need to update every contracts.
                            debug!(
                                "Ignored unknown DataType {:?} for compatible purpose.",
                                data_type_in_int
                            );
                        }
                    }

                    i += 1;
                }
                Err(SysError::IndexOutOfBound) => break,
                Err(e) => return Err(Error::from(e)),
            }
        }

        Ok(WitnessesParser {
            witnesses,
            configs: Configs::new(),
            dep: Vec::new(),
            old: Vec::new(),
            new: Vec::new(),
        })
    }

    pub fn parse_action_with_params(&self) -> Result<Option<(Bytes, Vec<Bytes>)>, Error> {
        if self.witnesses.is_empty() {
            return Ok(None);
        }

        let (index, data_type) = self.witnesses[0];
        let raw = util::load_das_witnesses(index, data_type)?;

        let action_data =
            ActionData::from_slice(raw.get(7..).unwrap()).map_err(|_| Error::WitnessActionDecodingError)?;
        let params = match action_data.as_reader().action().raw_data() {
            b"transfer_account"
            | b"edit_manager"
            | b"edit_records"
            | b"withdraw_from_wallet"
            | b"start_account_sale"
            | b"edit_account_sale"
            | b"cancel_account_sale"
            | b"start_account_auction"
            | b"edit_account_auction"
            | b"cancel_account_auction" => {
                if action_data.params().is_empty() {
                    return Err(Error::ParamsDecodingError);
                }
                vec![action_data.params()]
            }
            b"buy_account" => {
                let bytes = action_data.as_reader().params().raw_data();
                assert!(
                    bytes.len() > 4,
                    Error::InvalidTransactionStructure,
                    "The params of this action should contains two Script struct."
                );

                let length_of_inviter_lock = u32::from_le_bytes((&bytes[..4]).try_into().unwrap()) as usize;
                let bytes_of_inviter_lock = &bytes[..length_of_inviter_lock];
                let bytes_of_channel_lock = &bytes[length_of_inviter_lock..];
                // debug!("bytes_of_inviter_lock = 0x{}", util::hex_string(bytes_of_inviter_lock));
                // debug!("bytes_of_channel_lock = 0x{}", util::hex_string(bytes_of_channel_lock));

                vec![Bytes::from(bytes_of_inviter_lock), Bytes::from(bytes_of_channel_lock)]
            }
            _ => Vec::new(),
        };

        Ok(Some((action_data.action(), params)))
    }

    pub fn parse_config(&mut self, config_types: &[DataType]) -> Result<(), Error> {
        debug!("Parsing config witnesses only ...");

        // Filter out ConfigCells that have not been loaded yet. This only works for ConfigCells that maybe loaded multiple times.
        let unloaded_config_types = config_types
            .iter()
            .filter(|&&config_type| {
                // Skip some exists ConfigCells.
                match config_type {
                    DataType::ConfigCellMain => return self.configs.main().is_err(),
                    DataType::ConfigCellCharSetEmoji
                    | DataType::ConfigCellCharSetDigit
                    | DataType::ConfigCellCharSetEn
                    | DataType::ConfigCellCharSetZhHans
                    | DataType::ConfigCellCharSetZhHant => {
                        if self.configs.char_set().is_ok() {
                            let char_sets = self.configs.char_set().unwrap();
                            let char_set_index = das_types_util::data_type_to_char_set(config_type.to_owned());
                            return char_sets[char_set_index as usize].is_none();
                        }
                        return true;
                    }
                    _ => return true,
                }
            })
            .collect::<Vec<_>>();

        if unloaded_config_types.len() == 0 {
            debug!("  Skip all loaded ConfigCells ...");
            return Ok(());
        }

        debug!("  Load ConfigCells {:?} from cell_deps ...", unloaded_config_types);

        let config_cell_type = util::script_literal_to_script(CONFIG_CELL_TYPE);
        let mut config_data_types = Vec::new();
        let mut config_entity_hashes = map::Map::new();
        for config_type in unloaded_config_types {
            let args = Bytes::from((config_type.to_owned() as u32).to_le_bytes().to_vec());
            let type_script = config_cell_type.clone().as_builder().args(args.into()).build();
            // There must be one ConfigCell in the cell_deps, no more and no less.
            let ret = util::find_cells_by_script(ScriptType::Type, type_script.as_reader(), Source::CellDep)?;
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
                _ => {
                    warn!(
                        "  CellDeps[{}] Can not get entity hash from outputs_data.",
                        expected_cell_index
                    );
                    return Err(Error::InvalidCellData);
                }
            };

            // debug!(
            //     "    Load ConfigCell with DataType: {:?} Witness Hash: {:?}",
            //     config_type, expected_entity_hash
            // );

            // Store entity hash for later verification.
            config_entity_hashes.insert(expected_cell_index, expected_entity_hash);

            // Store data type for loading data on demand.
            config_data_types.push(config_type.to_owned())
        }

        debug!("  Load witnesses of the ConfigCells ...");

        macro_rules! assign_config_witness {
            ( $property:expr, $witness_type:ty, $entity:expr ) => {
                $property = Some(<$witness_type>::from_slice($entity).map_err(|e| {
                    warn!("Decoding witness error: {}", e.to_string());
                    Error::ConfigCellWitnessDecodingError
                })?)
            };
        }

        macro_rules! assign_config_char_set_witness {
            ( $char_set_type:expr, $entity:expr ) => {{
                let index = $char_set_type as usize;
                let char_set = CharSet {
                    name: $char_set_type,
                    // TODO make the meaning of following codes more clear
                    // skip 7 bytes das header, 4 bytes length
                    global: $entity.get(LENGTH_BYTES_4).unwrap() == &1u8,
                    data: $entity.get(5..).unwrap().to_vec(),
                };
                if self.configs.char_set.is_some() {
                    self.configs
                        .char_set
                        .as_mut()
                        .map(|char_sets| char_sets[index] = Some(char_set));
                } else {
                    let mut char_sets: Vec<Option<CharSet>> = vec![None; CHAR_SET_LENGTH];
                    char_sets[index] = Some(char_set);
                    self.configs.char_set = Some(char_sets)
                }
            }};
        }

        fn find_and_remove_from_hashes(
            _i: usize,
            _data_type: DataType,
            config_entity_hashes: &mut map::Map<usize, Vec<u8>>,
            entity: &[u8],
        ) -> Result<(), Error> {
            let entity_hash = util::blake2b_256(entity).to_vec();
            let ret = config_entity_hashes.find(&entity_hash).map(|v| v.to_owned());

            // debug!("current: 0x{}", util::hex_string(entity_hash.as_slice()));
            if let Some(key) = ret {
                // debug!("expected: 0x{}", util::hex_string(config_entity_hashes.get(&key).unwrap().as_slice()));
                config_entity_hashes.remove(&key);
            } else {
                // ⚠️ Do not print the whole entity, otherwise memory may be not enough.
                debug!(
                    "The witness of witness[{}] is corrupted! data_type: {:?} hash: 0x{} entity: {:?}",
                    _i,
                    _data_type,
                    util::hex_string(entity_hash.as_slice()),
                    entity.get(..40).map(|item| util::hex_string(item) + "...")
                );
                return Err(Error::ConfigCellWitnessIsCorrupted);
            }

            Ok(())
        }

        for (_i, witness_info) in self.witnesses.iter().enumerate() {
            let (index, data_type) = witness_info.to_owned();

            // Skip configs that no need to parse.
            if !config_data_types.contains(&data_type) {
                continue;
            }

            let raw = util::load_das_witnesses(index, data_type)?;
            let raw_trimmed = util::trim_empty_bytes(&raw);
            let entity = raw_trimmed.get(7..).ok_or(Error::ConfigCellWitnessDecodingError)?;

            find_and_remove_from_hashes(_i, data_type, &mut config_entity_hashes, entity)?;

            debug!(
                "  Found matched ConfigCell witness at: witnesses[{}] data_type: {:?} size: {}",
                _i,
                data_type,
                raw_trimmed.len()
            );

            match data_type {
                DataType::ConfigCellAccount => {
                    assign_config_witness!(self.configs.account, ConfigCellAccount, entity)
                }
                DataType::ConfigCellApply => {
                    assign_config_witness!(self.configs.apply, ConfigCellApply, entity)
                }
                DataType::ConfigCellIncome => {
                    assign_config_witness!(self.configs.income, ConfigCellIncome, entity)
                }
                DataType::ConfigCellMain => {
                    assign_config_witness!(self.configs.main, ConfigCellMain, entity)
                }
                DataType::ConfigCellPrice => {
                    assign_config_witness!(self.configs.price, ConfigCellPrice, entity)
                }
                DataType::ConfigCellProposal => {
                    assign_config_witness!(self.configs.proposal, ConfigCellProposal, entity)
                }
                DataType::ConfigCellProfitRate => {
                    assign_config_witness!(self.configs.profit_rate, ConfigCellProfitRate, entity)
                }
                DataType::ConfigCellRelease => {
                    assign_config_witness!(self.configs.release, ConfigCellRelease, entity)
                }
                DataType::ConfigCellSecondaryMarket => {
                    assign_config_witness!(self.configs.secondary_market, ConfigCellSecondaryMarket, entity)
                }
                DataType::ConfigCellRecordKeyNamespace => {
                    self.configs.record_key_namespace = Some(entity.get(LENGTH_BYTES_4..).unwrap().to_vec());
                }
                DataType::ConfigCellPreservedAccount00
                | DataType::ConfigCellPreservedAccount01
                | DataType::ConfigCellPreservedAccount02
                | DataType::ConfigCellPreservedAccount03
                | DataType::ConfigCellPreservedAccount04
                | DataType::ConfigCellPreservedAccount05
                | DataType::ConfigCellPreservedAccount06
                | DataType::ConfigCellPreservedAccount07
                | DataType::ConfigCellPreservedAccount08
                | DataType::ConfigCellPreservedAccount09
                | DataType::ConfigCellPreservedAccount10
                | DataType::ConfigCellPreservedAccount11
                | DataType::ConfigCellPreservedAccount12
                | DataType::ConfigCellPreservedAccount13
                | DataType::ConfigCellPreservedAccount14
                | DataType::ConfigCellPreservedAccount15
                | DataType::ConfigCellPreservedAccount16
                | DataType::ConfigCellPreservedAccount17
                | DataType::ConfigCellPreservedAccount18
                | DataType::ConfigCellPreservedAccount19 => {
                    // debug!("length: {}", entity.get(4..).unwrap().len());
                    // self.configs.preserved_account = None;
                    self.configs.preserved_account = Some(entity.get(LENGTH_BYTES_4..).unwrap().to_vec());
                }
                DataType::ConfigCellUnAvailableAccount => {
                    // debug!("length: {}", entity.get(LENGTH_BYTES_4..).unwrap().len());
                    self.configs.unavailable_account = Some(entity.get(LENGTH_BYTES_4..).unwrap().to_vec());
                }
                DataType::ConfigCellCharSetEmoji
                | DataType::ConfigCellCharSetDigit
                | DataType::ConfigCellCharSetEn
                | DataType::ConfigCellCharSetZhHans
                | DataType::ConfigCellCharSetZhHant => {
                    let char_set_type = das_types_util::data_type_to_char_set(data_type);
                    assign_config_char_set_witness!(char_set_type, entity)
                }
                _ => return Err(Error::ConfigTypeIsUndefined),
            }
        }

        // Check if there is any hash is not used, which means some config is missing.
        assert!(
            config_entity_hashes.is_empty(),
            Error::ConfigIsPartialMissing,
            "Can not find some ConfigCells' witnesses. (hashes: {:?})",
            config_entity_hashes
                .items
                .iter()
                .map(|(_, value)| util::hex_string(value.as_slice()))
                .collect::<Vec<String>>()
        );

        Ok(())
    }

    pub fn parse_cell(&mut self) -> Result<(), Error> {
        debug!("Parsing witnesses of all other cells ...");

        for (_i, witness) in self.witnesses.iter().enumerate() {
            let (index, data_type) = witness.to_owned();
            // Skip ActionData witness and ConfigCells' witnesses.
            if data_type == DataType::ActionData || self.is_config_data_type(data_type) {
                continue;
            }

            let raw = util::load_das_witnesses(index, data_type)?;

            // debug!("Parse witnesses[{}] in type: {:?}", _i, data_type);

            let data = Self::parse_data(raw.as_slice())?;
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

    fn parse_data(witness: &[u8]) -> Result<Data, Error> {
        // debug!(
        //     "witness[..3] = 0x{}",
        //     util::hex_string(witness.get(..3).unwrap())
        // );
        // debug!(
        //     "witness[3..7] = 0x{}",
        //     util::hex_string(witness.get(3..7).unwrap())
        // );

        if let Some(raw) = witness.get(HEADER_BYTES_7..HEADER_BYTES_7 + LENGTH_BYTES_4) {
            // Because of the redundancy of the witness, appropriate trimming is performed here.
            let length = u32::from_le_bytes(raw.try_into().unwrap()) as usize;

            // debug!(
            //     "witness[7..11] = 0x{}",
            //     util::hex_string(witness.get(7..11).unwrap())
            // );
            // debug!("stored data length: {}", length);
            // debug!("real data length: {}", witness.get(7..).unwrap().len());

            if let Some(raw) = witness.get(HEADER_BYTES_7..(HEADER_BYTES_7 + length)) {
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

        let unwrapped_entity = entity.as_reader().raw_data();
        let hash = util::blake2b_256(unwrapped_entity).to_vec();

        // debug!(
        //     "entity: index = {} hash = {:?} entity = {:?}",
        //     index, hash, unwrapped_entity
        // );

        Ok((index, version, data_type, hash, entity))
    }

    pub fn verify_and_get(&self, index: usize, source: Source) -> Result<(u32, DataType, &Bytes), Error> {
        let data = util::load_cell_data(index, source)?;
        let hash = match data.get(..32) {
            Some(bytes) => bytes.to_vec(),
            _ => {
                warn!("  {:?}[{}] Can not get entity hash from outputs_data.", source, index);
                return Err(Error::InvalidCellData);
            }
        };

        self.verify_with_hash_and_get(&hash, index, source)
    }

    pub fn verify_with_hash_and_get(
        &self,
        expected_hash: &[u8],
        index: usize,
        source: Source,
    ) -> Result<(u32, DataType, &Bytes), Error> {
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
            if expected_hash == _hash.as_slice() {
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
                    util::hex_string(expected_hash),
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
                util::hex_string(expected_hash)
            );
            return Err(Error::WitnessDataIndexMissMatch);
        }

        Ok((version, data_type, entity))
    }

    fn is_config_data_type(&self, data_type: DataType) -> bool {
        let config_data_types = [
            DataType::ConfigCellAccount,
            DataType::ConfigCellApply,
            DataType::ConfigCellIncome,
            DataType::ConfigCellMain,
            DataType::ConfigCellPrice,
            DataType::ConfigCellProposal,
            DataType::ConfigCellProfitRate,
            DataType::ConfigCellRelease,
            DataType::ConfigCellUnAvailableAccount,
            DataType::ConfigCellSecondaryMarket,
            DataType::ConfigCellRecordKeyNamespace,
            DataType::ConfigCellPreservedAccount00,
            DataType::ConfigCellPreservedAccount01,
            DataType::ConfigCellPreservedAccount02,
            DataType::ConfigCellPreservedAccount03,
            DataType::ConfigCellPreservedAccount04,
            DataType::ConfigCellPreservedAccount05,
            DataType::ConfigCellPreservedAccount06,
            DataType::ConfigCellPreservedAccount07,
            DataType::ConfigCellPreservedAccount08,
            DataType::ConfigCellPreservedAccount09,
            DataType::ConfigCellPreservedAccount10,
            DataType::ConfigCellPreservedAccount11,
            DataType::ConfigCellPreservedAccount12,
            DataType::ConfigCellPreservedAccount13,
            DataType::ConfigCellPreservedAccount14,
            DataType::ConfigCellPreservedAccount15,
            DataType::ConfigCellPreservedAccount16,
            DataType::ConfigCellPreservedAccount17,
            DataType::ConfigCellPreservedAccount18,
            DataType::ConfigCellPreservedAccount19,
            DataType::ConfigCellCharSetEmoji,
            DataType::ConfigCellCharSetDigit,
            DataType::ConfigCellCharSetEn,
            DataType::ConfigCellCharSetZhHans,
            DataType::ConfigCellCharSetZhHant,
        ];

        config_data_types.contains(&data_type)
    }
}
