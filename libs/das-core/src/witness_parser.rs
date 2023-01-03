use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::ToString;
use core::convert::{TryFrom, TryInto};

use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::syscalls;
use das_types::constants::{DataType, WITNESS_HEADER, WITNESS_HEADER_BYTES, WITNESS_LENGTH_BYTES, WITNESS_TYPE_BYTES};
use das_types::packed::*;
use das_types::prelude::*;

use super::constants::*;
use super::error::*;
use super::types::{Configs, LockScriptTypeIdTable};
use super::{assert, code_to_error, debug, util, warn};

#[derive(Debug)]
pub struct WitnessesParser {
    pub witnesses: Vec<(usize, DataType)>,
    pub configs: Configs,
    pub action: Vec<u8>,
    pub params: Vec<Bytes>,
    pub lock_type_id_table: LockScriptTypeIdTable,
    pub config_cell_type_id: Hash,
    // The Bytes is wrapped DataEntity.entity.
    dep: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
    old: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
    new: Vec<(u32, u32, DataType, Vec<u8>, Bytes)>,
}

impl WitnessesParser {
    fn is_config_data_type(data_type: &DataType) -> bool {
        let data_type_in_int = data_type.to_owned() as u32;
        data_type_in_int >= 100 && data_type_in_int <= 199999
    }

    pub fn new() -> Result<Self, Box<dyn ScriptError>> {
        let mut witnesses = Vec::new();
        let mut config_witnesses = BTreeMap::new();
        let mut found_config_cell_main = false;
        let mut i = 0;
        let mut das_witnesses_started = false;
        loop {
            let mut buf = [0u8; (WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)];
            let ret = syscalls::load_witness(&mut buf, 0, i, Source::Input);

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
                                    i, util::hex_string(&WITNESS_HEADER)
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

                    let data_type_in_int = u32::from_le_bytes(
                        buf.get(WITNESS_HEADER_BYTES..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES))
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    );
                    match DataType::try_from(data_type_in_int) {
                        Ok(DataType::SubAccount | DataType::SubAccountMintSign) => {
                            // Ignore sub-account witnesses in this parser.
                            debug!("witnesses[{:>2}] Found sub-account witness skip parsing.", i);
                        }
                        Ok(data_type) => {
                            if !das_witnesses_started {
                                assert!(
                                    data_type == DataType::ActionData,
                                    ErrorCode::WitnessStructureError,
                                    "The first DAS witness must be the type of DataType::ActionData ."
                                );
                                das_witnesses_started = true
                            }

                            if data_type == DataType::ConfigCellMain {
                                found_config_cell_main = true;
                            }

                            // If there is any ConfigCells in cell_deps, store its index and expected witness hash.
                            if Self::is_config_data_type(&data_type) {
                                debug!(
                                    "witnesses[{:>2}] Presume that the type of the witness is {:?} .",
                                    i, data_type
                                );

                                let args = Bytes::from((data_type.to_owned() as u32).to_le_bytes().to_vec());
                                let type_script = config_cell_type().as_builder().args(args.into()).build();
                                let config_cells = util::find_cells_by_script(
                                    ScriptType::Type,
                                    type_script.as_reader(),
                                    Source::CellDep,
                                )?;

                                if config_cells.len() > 0 {
                                    // For any type of ConfigCell, there should be one Cell in the cell_deps, no more and no less.
                                    assert!(
                                        config_cells.len() == 1,
                                        ErrorCode::ConfigCellIsRequired,
                                        "witnesses[{:>2}] There should be only one {:?} in cell_deps. (find_condition: {})",
                                        i,
                                        data_type,
                                        type_script
                                    );

                                    let data = util::load_cell_data(config_cells[0], Source::CellDep)?;
                                    assert!(
                                        data.len() >= 32,
                                        ErrorCode::WitnessStructureError,
                                        "witnesses[{:>2}] The witness of {:?} should have at least 32 bytes.",
                                        i,
                                        data_type
                                    );

                                    let mut expected_entity_hash = [0u8; 32];
                                    expected_entity_hash.copy_from_slice(&data);

                                    config_witnesses.insert(data_type as u32, (i, expected_entity_hash));
                                }
                            } else {
                                debug!(
                                    "witnesses[{:>2}] The type is {:?}, treat as non-config witness.",
                                    i, data_type
                                );
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
                Err(e) => return Err(Box::new(Error::<ErrorCode>::from(e))),
            }
        }

        let lock_type_id_table = LockScriptTypeIdTable {
            always_success: always_success_lock().into(),
            das_lock: das_lock().into(),
            secp256k1_blake160_signhash_all: signall_lock().into(),
            secp256k1_blake160_multisig_all: multisign_lock().into(),
        };

        Ok(WitnessesParser {
            witnesses,
            configs: Configs::new(config_witnesses),
            action: Vec::new(),
            params: Vec::new(),
            lock_type_id_table,
            config_cell_type_id: config_cell_type().code_hash().into(),
            dep: Vec::new(),
            old: Vec::new(),
            new: Vec::new(),
        })
    }

    pub fn parse_action_with_params(&mut self) -> Result<Option<(&[u8], &[Bytes])>, Box<dyn ScriptError>> {
        if self.witnesses.is_empty() {
            return Ok(None);
        }

        let (index, _) = self.witnesses[0];
        let raw = util::load_das_witnesses(index)?;

        let action_data = ActionData::from_slice(raw.get(7..).unwrap()).map_err(|e| {
            warn!(
                "witnesses[{:>2}] Decoding failed (expected to be ActionData): {}",
                index,
                e.to_string()
            );
            ErrorCode::WitnessActionDecodingError
        })?;
        let action = action_data.as_reader().action().raw_data().to_vec();

        let params = match action.as_slice() {
            b"buy_account" => {
                let bytes = action_data.as_reader().params().raw_data();
                let first_header = bytes.get(..4).ok_or(ErrorCode::ParamsDecodingError)?;
                let length_of_inviter_lock = u32::from_le_bytes(first_header.try_into().unwrap()) as usize;
                let bytes_of_inviter_lock = bytes
                    .get(..length_of_inviter_lock)
                    .ok_or(ErrorCode::ParamsDecodingError)?;

                let second_header = bytes
                    .get(length_of_inviter_lock..(length_of_inviter_lock + 4))
                    .ok_or(ErrorCode::ParamsDecodingError)?;
                let length_of_channel_lock = u32::from_le_bytes(second_header.try_into().unwrap()) as usize;
                let bytes_of_channel_lock = bytes
                    .get(length_of_inviter_lock..(length_of_inviter_lock + length_of_channel_lock))
                    .ok_or(ErrorCode::ParamsDecodingError)?;
                let bytes_of_role = bytes
                    .get((length_of_inviter_lock + length_of_channel_lock)..)
                    .ok_or(ErrorCode::ParamsDecodingError)?;

                assert!(
                    bytes_of_role.len() == 1,
                    ErrorCode::ParamsDecodingError,
                    "The params of this action should contains a param of role at the end."
                );

                // debug!("bytes_of_inviter_lock = 0x{}", util::hex_string(bytes_of_inviter_lock));
                // debug!("bytes_of_channel_lock = 0x{}", util::hex_string(bytes_of_channel_lock));

                vec![
                    Bytes::from(bytes_of_inviter_lock),
                    Bytes::from(bytes_of_channel_lock),
                    Bytes::from(bytes_of_role),
                ]
            }
            b"lock_account_for_cross_chain" => {
                let bytes = action_data.as_reader().params().raw_data();

                assert!(
                    bytes.len() == 8 + 8 + 1,
                    ErrorCode::ParamsDecodingError,
                    "The params of this action should contains 8 bytes coin_type, 8 bytes chain_id and 1 byte role."
                );

                let coin_type = &bytes[0..8];
                let chain_id = &bytes[8..16];
                let role = bytes[16];

                vec![
                    Bytes::from(coin_type),
                    Bytes::from(chain_id),
                    Bytes::from(vec![role].as_slice()),
                ]
            }
            _ => {
                if action_data.params().is_empty() {
                    Vec::new()
                } else {
                    vec![action_data.params()]
                }
            }
        };

        self.action = action;
        self.params = params;

        Ok(Some((&self.action, &self.params)))
    }

    pub fn get_lock_script_type(&self, script_reader: ScriptReader) -> Option<LockScript> {
        match script_reader {
            x if util::is_type_id_equal(self.lock_type_id_table.always_success.as_reader().into(), x.into()) => {
                Some(LockScript::AlwaysSuccessLock)
            }
            x if util::is_type_id_equal(self.lock_type_id_table.das_lock.as_reader().into(), x.into()) => {
                Some(LockScript::DasLock)
            }
            x if util::is_type_id_equal(
                self.lock_type_id_table
                    .secp256k1_blake160_signhash_all
                    .as_reader()
                    .into(),
                x.into(),
            ) =>
            {
                Some(LockScript::Secp256k1Blake160SignhashLock)
            }
            x if util::is_type_id_equal(
                self.lock_type_id_table
                    .secp256k1_blake160_multisig_all
                    .as_reader()
                    .into(),
                x.into(),
            ) =>
            {
                Some(LockScript::Secp256k1Blake160MultisigLock)
            }
            _ => None,
        }
    }

    pub fn get_type_script_type(&self, script_reader: ScriptReader) -> Option<TypeScript> {
        if script_reader.hash_type().as_slice()[0] != ScriptHashType::Type as u8 {
            return None;
        }

        let type_id_table_reader = self
            .configs
            .main()
            .expect("Expect ConfigCellMain has been loaded.")
            .type_id_table();

        match script_reader.code_hash() {
            x if util::is_reader_eq(x, type_id_table_reader.apply_register_cell()) => {
                Some(TypeScript::ApplyRegisterCellType)
            }
            x if util::is_reader_eq(x, type_id_table_reader.account_cell()) => Some(TypeScript::AccountCellType),
            x if util::is_reader_eq(x, type_id_table_reader.account_sale_cell()) => {
                Some(TypeScript::AccountSaleCellType)
            }
            x if util::is_reader_eq(x, type_id_table_reader.account_auction_cell()) => {
                Some(TypeScript::AccountAuctionCellType)
            }
            x if util::is_reader_eq(x, type_id_table_reader.balance_cell()) => Some(TypeScript::BalanceCellType),
            x if util::is_reader_eq(x, type_id_table_reader.income_cell()) => Some(TypeScript::IncomeCellType),
            x if util::is_reader_eq(x, type_id_table_reader.offer_cell()) => Some(TypeScript::OfferCellType),
            x if util::is_reader_eq(x, type_id_table_reader.pre_account_cell()) => Some(TypeScript::PreAccountCellType),
            x if util::is_reader_eq(x, type_id_table_reader.proposal_cell()) => Some(TypeScript::ProposalCellType),
            x if util::is_reader_eq(x, type_id_table_reader.reverse_record_cell()) => {
                Some(TypeScript::ReverseRecordCellType)
            }
            x if util::is_reader_eq(x, type_id_table_reader.sub_account_cell()) => Some(TypeScript::SubAccountCellType),
            x if util::is_reader_eq(x, self.config_cell_type_id.as_reader()) => Some(TypeScript::ConfigCellType),
            _ => None,
        }
    }

    pub fn parse_cell(&mut self) -> Result<(), Box<dyn ScriptError>> {
        debug!("Parsing witnesses of all other cells ...");
        // witness format 1: 'das'(3) + DATA_TYPE(4) + molecule

        for (_i, witness) in self.witnesses.iter().enumerate() {
            let (index, data_type) = witness.to_owned();
            // Skip ActionData witness and ConfigCells' witnesses.
            if data_type == DataType::ActionData || Self::is_config_data_type(&data_type) {
                continue;
            }

            let raw = util::load_das_witnesses(index)?;

            let data = Self::parse_data(raw.as_slice())?;
            let mut _cell_index = 0;
            if let Some(entity) = data.dep().to_opt() {
                let entity_info = Self::parse_entity(entity, data_type)?;
                _cell_index = entity_info.0;
                self.dep.push(entity_info)
            }
            if let Some(entity) = data.old().to_opt() {
                let entity_info = Self::parse_entity(entity, data_type)?;
                _cell_index = entity_info.0;
                self.old.push(entity_info)
            }
            if let Some(entity) = data.new().to_opt() {
                let entity_info = Self::parse_entity(entity, data_type)?;
                _cell_index = entity_info.0;
                self.new.push(entity_info)
            }

            #[cfg(all(debug_assertions))]
            {
                let mut source = None;
                if let Some(_) = data.dep().to_opt() {
                    source = Some(Source::CellDep);
                }
                if let Some(_) = data.old().to_opt() {
                    source = Some(Source::Input);
                }
                if let Some(_) = data.new().to_opt() {
                    source = Some(Source::Output);
                }
                debug!(
                    "  witnesses[{:>2}] {{ data_type: {:?}, source: {:?}, index: {} }}",
                    index, data_type, source, _cell_index
                );
            }
        }

        Ok(())
    }

    fn parse_data(witness: &[u8]) -> Result<Data, Box<dyn ScriptError>> {
        // debug!(
        //     "witness[..3] = 0x{}",
        //     util::hex_string(witness.get(..3).unwrap())
        // );
        // debug!(
        //     "witness[3..7] = 0x{}",
        //     util::hex_string(witness.get(3..7).unwrap())
        // );

        if let Some(raw) = witness.get(
            (WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)
                ..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES + WITNESS_LENGTH_BYTES),
        ) {
            // Because of the redundancy of the witness, appropriate trimming is performed here.
            let length = u32::from_le_bytes(raw.try_into().unwrap()) as usize;

            // debug!("witness[7..11] = 0x{}", util::hex_string(witness.get(7..11).unwrap()));
            // debug!("stored data length: {}", length);
            // debug!("real data length: {}", witness.get(7..).unwrap().len());

            if let Some(raw) = witness
                .get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES + length))
            {
                let data = match Data::from_slice(raw) {
                    Ok(data) => data,
                    Err(_e) => {
                        debug!("WitnessDataDecodingError: {:?}", _e);
                        return Err(code_to_error!(ErrorCode::WitnessDataDecodingError));
                    }
                };
                Ok(data)
            } else {
                Err(code_to_error!(ErrorCode::WitnessDataReadDataBodyFailed))
            }
        } else {
            Err(code_to_error!(ErrorCode::WitnessDataParseLengthHeaderFailed))
        }
    }

    fn parse_entity(
        data_entity: DataEntity,
        data_type: DataType,
    ) -> Result<(u32, u32, DataType, Vec<u8>, Bytes), Box<dyn ScriptError>> {
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

    fn get(
        &self,
        index: u32,
        source: Source,
    ) -> Result<Option<&(u32, u32, DataType, Vec<u8>, Bytes)>, Box<dyn ScriptError>> {
        let group = match source {
            Source::Input => &self.old,
            Source::Output => &self.new,
            Source::CellDep => &self.dep,
            _ => {
                return Err(code_to_error!(ErrorCode::HardCodedError));
            }
        };

        Ok(group.iter().find(|&(i, _, _, _, _)| *i == index))
    }

    pub fn verify_hash(&self, index: usize, source: Source) -> Result<(), Box<dyn ScriptError>> {
        let data = util::load_cell_data(index, source)?;
        let expected_hash = match data.get(..32) {
            Some(bytes) => bytes,
            _ => {
                warn!("  {:?}[{}] Can not get entity hash from outputs_data.", source, index);
                return Err(code_to_error!(ErrorCode::InvalidCellData));
            }
        };

        if let Some((_, _, _, _hash, _)) = self.get(index as u32, source)? {
            assert!(
                expected_hash == _hash,
                ErrorCode::WitnessDataHashOrTypeMissMatch,
                "{:?}[{}] Can not find witness.(expected_hash: 0x{}, current_hash: 0x{})",
                source,
                index,
                util::hex_string(expected_hash),
                util::hex_string(_hash)
            );
        } else {
            // This error means the there is no witness.data.dep/old/new.index matches the index of the cell.
            warn!(
                "{:?}[{}] Can not find witness.(expected_hash: 0x{})",
                source,
                index,
                util::hex_string(expected_hash)
            );
            return Err(code_to_error!(ErrorCode::WitnessDataIndexMissMatch));
        }

        Ok(())
    }

    pub fn verify_and_get(
        &self,
        data_type: DataType,
        index: usize,
        source: Source,
    ) -> Result<(u32, DataType, &Bytes), Box<dyn ScriptError>> {
        let data = match util::load_cell_data(index, source) {
            Ok(data) => data,
            _ => {
                debug!("  {:?}[{}] Can not get outputs_data.", source, index);
                return Err(code_to_error!(ErrorCode::InvalidCellData));
            }
        };
        let hash = match data.get(..32) {
            Some(bytes) => bytes,
            _ => {
                warn!("  {:?}[{}] Can not get entity hash from outputs_data.", source, index);
                return Err(code_to_error!(ErrorCode::InvalidCellData));
            }
        };

        self.verify_with_hash_and_get(hash, data_type, index, source)
    }

    pub fn verify_with_hash_and_get(
        &self,
        expected_hash: &[u8],
        data_type: DataType,
        index: usize,
        source: Source,
    ) -> Result<(u32, DataType, &Bytes), Box<dyn ScriptError>> {
        let version;
        let entity;
        if let Some((_, _version, _data_type, _hash, _entity)) = self.get(index as u32, source)? {
            if expected_hash == _hash.as_slice() && &data_type == _data_type {
                version = _version.to_owned();
                entity = _entity;
            } else {
                // This error means the there is no hash(witness.data.dep/old/new.entity) matches the leading 32 bytes of the cell.
                debug!(
                    "  {:?}[{}] Witness hash or data_type verification failed: expected_data_type: {:?}, witness_data_type: {:?}, hash_in_cell_data: 0x{} calculated_hash: 0x{} entity: 0x{}",
                    source,
                    index,
                    data_type,
                    _data_type,
                    util::hex_string(expected_hash),
                    util::hex_string(_hash.as_slice()),
                    util::hex_string(_entity.as_reader().raw_data())
                );
                return Err(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch));
            }
        } else {
            // This error means the there is no witness.data.dep/old/new.index matches the index of the cell.
            warn!(
                "Can not find witness at {:?}[{}], expected hash: 0x{}",
                source,
                index,
                util::hex_string(expected_hash)
            );
            return Err(code_to_error!(ErrorCode::WitnessDataIndexMissMatch));
        }

        Ok((version, data_type, entity))
    }
}
