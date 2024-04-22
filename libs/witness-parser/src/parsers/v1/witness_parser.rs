#[cfg(feature = "no_std")]
use alloc::collections::btree_map::BTreeMap;
#[cfg(feature = "no_std")]
use alloc::format;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use core::cell::OnceCell;
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::format;

#[cfg(feature = "no_std")]
use ckb_std::error::SysError;
#[cfg(feature = "no_std")]
use ckb_std::syscalls;
use das_types::constants::{
    config_cell_type, Action, ActionParams, DataType, Source, TypeScript, WITNESS_HEADER, WITNESS_HEADER_BYTES,
    WITNESS_TYPE_BYTES,
};
use das_types::prelude::*;
use das_types::{packed, util as types_util};

use super::action_parser::parse_action;
use crate::constants::ScriptType;
use crate::error::WitnessParserError;
use crate::traits::WitnessQueryable;
use crate::types::{CellMeta, Hash, WitnessMeta};
use crate::util;

#[derive(Debug, Default)]
pub struct WitnessesParser {
    pub action: Action,
    pub action_params: ActionParams,
    pub action_data: packed::ActionData,

    inited: bool,
    witnesses: Vec<WitnessMeta>,
    cell_meta_map: BTreeMap<CellMeta, usize>,
    data_type_map: BTreeMap<DataType, usize>,
}

impl WitnessesParser {
    pub fn get_instance() -> &'static mut Self {
        static mut WITNESS_PARSER: OnceCell<WitnessesParser> = OnceCell::new();
        unsafe {
            WITNESS_PARSER.get_or_init(|| {
                let res = Self::default();
                // TODO Try a better way to implement singleton to handle the init errors.
                // res.init().unwrap();
                res
            });
            WITNESS_PARSER.get_mut().unwrap()
        }
    }

    pub fn init(&mut self) -> Result<(), WitnessParserError> {
        let mut das_witnesses_started = false;
        let mut i = 0;

        debug!("=== Init witness parser ===");

        loop {
            let mut buf = [0u8; (WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)];
            // TODO Replace all syscalls with tx-resolver to support std environment.
            let ret = syscalls::load_witness(&mut buf, 0, i, Source::Input.into());

            match ret {
                // Data which length is too short to be DAS witnesses, so ignore it.
                Ok(_) => i += 1,
                Err(SysError::LengthNotEnough(_)) => {
                    if let Some(raw) = buf.get(..WITNESS_HEADER_BYTES) {
                        if das_witnesses_started {
                            // If it is parsing DAS witnesses currently, end the parsing.
                            if raw != &WITNESS_HEADER {
                                debug!(
                                    "witnesses[{:>2}] Found witness not started with 0x{}, stop parsing the remain witnesses.",
                                    i, hex::encode(&WITNESS_HEADER)
                                );
                                break;
                            }
                        } else {
                            // If it is not parsing DAS witnesses currently, continue to detect the next witness.
                            if raw != &WITNESS_HEADER {
                                i += 1;
                                continue;
                            }
                        }
                    }

                    match util::parse_date_type_from_witness(i, &buf) {
                        Ok(data_type) => {
                            if !das_witnesses_started {
                                err_assert!(
                                    data_type == DataType::ActionData,
                                    WitnessParserError::OrderError {
                                        index: i,
                                        msg: String::from(
                                            "The first DAS witness must be the type of DataType::ActionData ."
                                        )
                                    }
                                );
                                das_witnesses_started = true
                            }

                            match data_type {
                                DataType::ActionData => {
                                    self.parse_action(i)?;
                                }
                                x if types_util::is_sub_account_data_type(&x) => {
                                    debug!("  witnesses[{:>2}] Found {:?} witness skip parsing.", i, x);
                                }
                                x if types_util::is_config_data_type(&x) => {
                                    self.push_witness_wrap_in_config(i, x)?;
                                }
                                x if types_util::is_other_data_type(&x) => {
                                    self.push_witness_for_other_data_type(i, x)?;
                                }

                                _ => {
                                    self.push_witness_wrap_in_data(i, data_type)?;
                                }
                            }
                        }
                        Err(WitnessParserError::UndefinedDataType {
                            index: _index,
                            date_type: _date_type,
                        }) => {
                            // Ignore unknown DataTypes which will make adding new DataType much easier and no need to update every contracts.
                            debug!(
                                "witnesses[{:>2}] Ignored unknown DataType {:?} for compatible purpose.",
                                _index, _date_type
                            );
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    };

                    i += 1;
                }
                Err(SysError::IndexOutOfBound) => break,
                Err(e) => return Err(WitnessParserError::SysError { index: i, err: e }),
            }
        }

        debug!("=== Witness parser inited ===");

        self.inited = true;
        Ok(())
    }

    pub fn is_inited(&self) -> bool {
        self.inited
    }

    pub fn get_action_data(&self) -> &packed::ActionData {
        &self.action_data
    }

    fn push_witness_wrap_in_config(&mut self, index: usize, data_type: DataType) -> Result<(), WitnessParserError> {
        debug!(
            "  witnesses[{:>2}] Presume that the type of the witness is {:?} .",
            index, data_type
        );

        let mut found_config_cell = false;
        for source in [Source::CellDep, Source::Input, Source::Output] {
            let args = packed::Bytes::from((data_type.to_owned() as u32).to_le_bytes().to_vec());
            let type_script = config_cell_type().clone().as_builder().args(args).build();
            let config_cells = util::find_cells_by_script(index, ScriptType::Type, type_script.as_reader(), source)?;

            // For any type of ConfigCell, there should be one Cell in the cell_deps, no more and no less.
            match config_cells.len() {
                0 => continue,
                1 => {
                    found_config_cell = true;
                }
                _ => return Err(WitnessParserError::DuplicatedConfigCellFound { index, data_type }),
            }

            let cell_index = config_cells[0];
            let hash_in_cell_data = Self::load_witness_hash_from_cell(index, cell_index, source)?;

            debug!(
                "  witnesses[{:>2}] {{ data_type: {:?}, index: {}, source: {:?}, hash_in_cell: {} }}",
                index,
                data_type,
                cell_index,
                source,
                hex::encode(&hash_in_cell_data)
            );

            self.data_type_map.insert(data_type, self.witnesses.len());
            self.witnesses.push(WitnessMeta {
                index,
                version: 0,
                data_type,
                cell_meta: CellMeta {
                    index: cell_index,
                    source: source,
                },
                hash_in_cell_data,
            });
        }

        err_assert!(
            found_config_cell,
            WitnessParserError::ConfigCellNotFound { index, data_type }
        );

        Ok(())
    }

    fn push_witness_wrap_in_data(&mut self, index: usize, data_type: DataType) -> Result<(), WitnessParserError> {
        debug!(
            "  witnesses[{:>2}] Presume that the type of the witness is {:?} .",
            index, data_type
        );

        let buf = util::load_das_witnesses(index)?;
        let data = util::parse_data_from_witness(index, &buf)?;

        let mut entities = vec![];
        let mut found_entity = false;
        if data.dep().to_opt().is_some() {
            let source = Source::CellDep;
            let data_entity = data.dep().to_opt().unwrap();
            entities.push((source, data_entity));
            found_entity = true;
        }
        if data.old().to_opt().is_some() {
            let source = Source::Input;
            let data_entity = data.old().to_opt().unwrap();
            entities.push((source, data_entity));
            found_entity = true;
        }
        if data.new().to_opt().is_some() {
            let source = Source::Output;
            let data_entity = data.new().to_opt().unwrap();
            entities.push((source, data_entity));
            found_entity = true;
        }

        err_assert!(
            found_entity,
            WitnessParserError::DecodingDataFailed {
                index,
                err: String::from("The witness should contains at least one of dep/old/new."),
            }
        );

        for (source, data_entity) in entities {
            let cell_index = u32::from(data_entity.index()) as usize;
            let version = u32::from(data_entity.version());
            let hash_in_cell_data = Self::load_witness_hash_from_cell(index, cell_index, source)?;

            debug!(
                "  witnesses[{:>2}] {{ data_type: {:?}, index: {}, source: {:?}, hash_in_cell: {} }}",
                index,
                data_type,
                cell_index,
                source,
                hex::encode(&hash_in_cell_data)
            );

            self.cell_meta_map.insert(
                CellMeta {
                    index: cell_index,
                    source,
                },
                self.witnesses.len(),
            );
            self.witnesses.push(WitnessMeta {
                index,
                version,
                data_type,
                cell_meta: CellMeta {
                    index: cell_index,
                    source,
                },
                hash_in_cell_data,
            });
        }

        Ok(())
    }
    fn push_witness_for_other_data_type(
        &mut self,
        index: usize,
        data_type: DataType,
    ) -> Result<(), WitnessParserError> {
        debug!(
            "  witnesses[{:>2}] Presume that the type of the witness is {:?} .",
            index, data_type
        );

        let buf = util::load_das_witnesses(index)?;
        let hash_in_cell_data =
            types_util::blake2b_256(buf.get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..).unwrap());
        debug!(
            "  witnesses[{:>2}] {{ data_type: {:?}, hash_in_cell: {} }}",
            index,
            data_type,
            hex::encode(&hash_in_cell_data)
        );
        self.witnesses.push(WitnessMeta {
            index,
            version: 0,
            data_type,
            cell_meta: CellMeta {
                //todo: 255 and CellDep is a magic value, need to be replaced by a proper value
                index: 255,
                source: Source::CellDep,
            },
            hash_in_cell_data,
        });

        Ok(())
    }
    fn load_witness_hash_from_cell(
        witness_index: usize,
        cell_index: usize,
        source: Source,
    ) -> Result<[u8; 32], WitnessParserError> {
        let data = util::load_cell_data(cell_index, source.into())?;
        debug!(
            "  witnesses[{:>2}] Loading expected hash from {:?}[{}]",
            witness_index, source, cell_index
        );

        err_assert!(
            data.len() >= 32,
            WitnessParserError::CanNotGetVerficicationHashFromCell {
                index: witness_index,
                msg: format!(
                    "The outputs_data of {:?}[{}] should have at least 32 bytes.",
                    source, cell_index
                )
            }
        );

        let mut expected_entity_hash = [0u8; 32];
        expected_entity_hash.copy_from_slice(&data[..32]);

        Ok(expected_entity_hash)
    }

    fn parse_action(&mut self, index: usize) -> Result<(), WitnessParserError> {
        let bytes = util::load_das_witnesses(index)?;
        let (action_data, action, action_params) = parse_action(index, bytes)?;
        self.action_data = action_data;
        self.action = action;
        self.action_params = action_params;

        Ok(())
    }
}

impl WitnessQueryable for WitnessesParser {
    fn get_witness_meta_by_index(&mut self, index: usize) -> Result<WitnessMeta, WitnessParserError> {
        err_assert!(self.inited, WitnessParserError::InitializationRequired);

        let witness = self
            .witnesses
            .get(index)
            .ok_or(WitnessParserError::CanNotFindWitnessByIndex { index })?;

        Ok(witness.to_owned())
    }

    fn get_witness_meta_by_cell_meta(&mut self, cell_meta: CellMeta) -> Result<WitnessMeta, WitnessParserError> {
        err_assert!(self.inited, WitnessParserError::InitializationRequired);

        let index = self
            .cell_meta_map
            .get(&cell_meta)
            .ok_or(WitnessParserError::CanNotFindWitnessByCellMeta {
                source: cell_meta.source,
                index: cell_meta.index,
            })?
            .to_owned();

        self.get_witness_meta_by_index(index)
    }

    fn get_type_id(&mut self, type_script: TypeScript) -> Result<Hash, WitnessParserError> {
        err_assert!(self.inited, WitnessParserError::InitializationRequired);

        let config_cell_type = config_cell_type();
        let config = self.get_entity_by_data_type::<packed::ConfigCellMain>(DataType::ConfigCellMain)?;
        let type_id = match type_script {
            TypeScript::AccountCellType => config.type_id_table().account_cell(),
            TypeScript::AccountSaleCellType => config.type_id_table().account_sale_cell(),
            TypeScript::AccountAuctionCellType => config.type_id_table().account_auction_cell(),
            TypeScript::ApplyRegisterCellType => config.type_id_table().apply_register_cell(),
            TypeScript::BalanceCellType => config.type_id_table().balance_cell(),
            TypeScript::ConfigCellType => config_cell_type.code_hash(),
            TypeScript::IncomeCellType => config.type_id_table().income_cell(),
            TypeScript::OfferCellType => config.type_id_table().offer_cell(),
            TypeScript::PreAccountCellType => config.type_id_table().pre_account_cell(),
            TypeScript::ProposalCellType => config.type_id_table().proposal_cell(),
            TypeScript::ReverseRecordCellType => config.type_id_table().reverse_record_cell(),
            TypeScript::SubAccountCellType => config.type_id_table().sub_account_cell(),
            TypeScript::ReverseRecordRootCellType => config.type_id_table().reverse_record_root_cell(),
            TypeScript::DPointCellType => config.type_id_table().dpoint_cell(),
            TypeScript::EIP712Lib => config.type_id_table().eip712_lib(),
            TypeScript::DeviceKeyListCellType => config.type_id_table().key_list_config_cell(),
        };

        let type_id_vec = type_id.as_slice().to_vec();
        let mut ret = Hash::default();
        ret.copy_from_slice(&type_id_vec);

        Ok(ret)
    }

    fn get_entity_by_cell_meta<T: Entity>(&mut self, cell_meta: CellMeta) -> Result<T, WitnessParserError> {
        err_assert!(self.inited, WitnessParserError::InitializationRequired);

        let index = self
            .cell_meta_map
            .get(&cell_meta)
            .ok_or(WitnessParserError::CanNotFindWitnessByCellMeta {
                source: cell_meta.source,
                index: cell_meta.index,
            })?
            .to_owned();

        let witness_meta = self
            .witnesses
            .get(index)
            .ok_or(WitnessParserError::CanNotFindWitnessByIndex { index })?;

        let buf = util::load_das_witnesses(witness_meta.index)?;
        let data = util::parse_data_from_witness(index, &buf)?;
        let data_entity = match witness_meta.cell_meta.source {
            Source::CellDep => data.dep().to_opt(),
            Source::Input => data.old().to_opt(),
            Source::Output => data.new().to_opt(),
        }
        .ok_or(WitnessParserError::DecodingDataFailed {
            index,
            err: String::from("The witness.data should contains at least one of dep/old/new."),
        })?;

        let entity_hash = types_util::blake2b_256(data_entity.as_reader().entity().raw_data());
        err_assert!(
            witness_meta.hash_in_cell_data == entity_hash,
            WitnessParserError::WitnessHashMismatched {
                index: witness_meta.index,
                in_cell_data: hex::encode(&witness_meta.hash_in_cell_data),
                actual: hex::encode(&entity_hash)
            }
        );

        let data_type = util::parse_date_type_from_witness(index, &buf)?;

        let version = u32::from(data_entity.version());

        let entity = T::from_compatible_slice(data_entity.as_reader().entity().raw_data()).map_err(|_err| {
            WitnessParserError::DecodingEntityFailed {
                index,
                data_type,
                version,
            }
        })?;

        Ok(entity)
    }

    fn get_entity_by_data_type<T: Entity>(&mut self, data_type: DataType) -> Result<T, WitnessParserError> {
        err_assert!(self.inited, WitnessParserError::InitializationRequired);

        let index = self
            .data_type_map
            .get(&data_type)
            .ok_or(WitnessParserError::CanNotFindWitnessByDataType { data_type })?
            .to_owned();

        let witness_meta = self
            .witnesses
            .get(index)
            .ok_or(WitnessParserError::CanNotFindWitnessByIndex { index })?;

        let buf = util::load_das_witnesses(witness_meta.index)?;
        let data = util::parse_raw_from_witness(index, &buf)?;

        let entity_hash = types_util::blake2b_256(&data);
        err_assert!(
            witness_meta.hash_in_cell_data == entity_hash,
            WitnessParserError::WitnessHashMismatched {
                index: witness_meta.index,
                in_cell_data: hex::encode(&witness_meta.hash_in_cell_data),
                actual: hex::encode(&entity_hash)
            }
        );

        let data_type = util::parse_date_type_from_witness(index, &buf)?;
        let entity = T::from_compatible_slice(&data).map_err(|_err| WitnessParserError::DecodingEntityFailed {
            index,
            data_type,
            version: 0,
        })?;

        Ok(entity)
    }

    fn get_raw_by_index(&mut self, index: usize) -> Result<Vec<u8>, WitnessParserError> {
        err_assert!(self.inited, WitnessParserError::InitializationRequired);

        let witness_meta = self
            .witnesses
            .get(index)
            .ok_or(WitnessParserError::CanNotFindWitnessByIndex { index })?;

        let buf = util::load_das_witnesses(witness_meta.index)?;

        let buf_hash = types_util::blake2b_256(buf.get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..).unwrap());
        err_assert!(
            witness_meta.hash_in_cell_data == buf_hash,
            WitnessParserError::WitnessHashMismatched {
                index: witness_meta.index,
                in_cell_data: hex::encode(&witness_meta.hash_in_cell_data),
                actual: hex::encode(&buf_hash)
            }
        );

        Ok(buf)
    }

    fn get_raw_by_cell_meta(&mut self, cell_meta: CellMeta) -> Result<Vec<u8>, WitnessParserError> {
        let index = self
            .cell_meta_map
            .get(&cell_meta)
            .ok_or(WitnessParserError::CanNotFindWitnessByCellMeta {
                source: cell_meta.source,
                index: cell_meta.index,
            })?
            .to_owned();

        self.get_raw_by_index(index)
    }

    fn get_raw_by_data_type(&mut self, data_type: DataType) -> Result<Vec<u8>, WitnessParserError> {
        let index = self
            .data_type_map
            .get(&data_type)
            .ok_or(WitnessParserError::CanNotFindWitnessByDataType { data_type })?
            .to_owned();

        self.get_raw_by_index(index)
    }
}
