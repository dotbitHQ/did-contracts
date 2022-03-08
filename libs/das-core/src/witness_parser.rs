use super::{
    assert,
    constants::*,
    debug,
    error::Error,
    types::{CharSet, Configs, LockScriptTypeIdTable},
    util, warn,
};
use alloc::{
    collections::btree_map::BTreeMap,
    format,
    string::{String, ToString},
};
use ckb_std::{ckb_constants::Source, error::SysError, syscalls};
use core::convert::{TryFrom, TryInto};
use das_types::constants::WITNESS_LENGTH_BYTES;
use das_types::{
    constants::{DataType, CHAR_SET_LENGTH, WITNESS_HEADER, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES},
    packed::*,
    prelude::*,
    util as das_types_util,
};

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
    pub fn new() -> Result<Self, Error> {
        let mut witnesses = Vec::new();
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

                    let data_type_in_int = u32::from_le_bytes(
                        buf.get(WITNESS_HEADER_BYTES..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES))
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    );
                    match DataType::try_from(data_type_in_int) {
                        Ok(DataType::SubAccount) => {
                            // Ignore sub-account witnesses in this parser.
                        }
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

        let lock_type_id_table = LockScriptTypeIdTable {
            always_success: always_success_lock().into(),
            das_lock: das_lock().into(),
            secp256k1_blake160_signhash_all: signall_lock().into(),
            secp256k1_blake160_multisig_all: multisign_lock().into(),
        };

        Ok(WitnessesParser {
            witnesses,
            configs: Configs::new(),
            action: Vec::new(),
            params: Vec::new(),
            lock_type_id_table,
            config_cell_type_id: config_cell_type().code_hash().into(),
            dep: Vec::new(),
            old: Vec::new(),
            new: Vec::new(),
        })
    }

    pub fn parse_action_with_params(&mut self) -> Result<Option<(&[u8], &[Bytes])>, Error> {
        if self.witnesses.is_empty() {
            return Ok(None);
        }

        let (index, _) = self.witnesses[0];
        let raw = util::load_das_witnesses(index)?;

        let action_data = ActionData::from_slice(raw.get(7..).unwrap()).map_err(|e| {
            warn!(
                "Decoding witnesses[{}](expected to be ActionData) failed: {}",
                index,
                e.to_string()
            );
            Error::WitnessActionDecodingError
        })?;
        let action = action_data.as_reader().action().raw_data().to_vec();

        let params = match action.as_slice() {
            b"buy_account" => {
                let bytes = action_data.as_reader().params().raw_data();
                let first_header = bytes.get(..4).ok_or(Error::ParamsDecodingError)?;
                let length_of_inviter_lock = u32::from_le_bytes(first_header.try_into().unwrap()) as usize;
                let bytes_of_inviter_lock = bytes.get(..length_of_inviter_lock).ok_or(Error::ParamsDecodingError)?;

                let second_header = bytes
                    .get(length_of_inviter_lock..(length_of_inviter_lock + 4))
                    .ok_or(Error::ParamsDecodingError)?;
                let length_of_channel_lock = u32::from_le_bytes(second_header.try_into().unwrap()) as usize;
                let bytes_of_channel_lock = bytes
                    .get(length_of_inviter_lock..(length_of_inviter_lock + length_of_channel_lock))
                    .ok_or(Error::ParamsDecodingError)?;
                let bytes_of_role = bytes
                    .get((length_of_inviter_lock + length_of_channel_lock)..)
                    .ok_or(Error::ParamsDecodingError)?;

                assert!(
                    bytes_of_role.len() == 1,
                    Error::ParamsDecodingError,
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

    pub fn parse_config(&mut self, config_types: &[DataType]) -> Result<(), Error> {
        debug!("Parsing config witnesses only ...");
        // entity witness 2: 'das'(3) + DATA_TYPE(4) + binary_data(molecule like data: LENGTH(4) + ENTITY)

        // Filter out ConfigCells that have not been loaded yet. This only works for ConfigCells that maybe loaded multiple times.
        let unloaded_config_types = config_types
            .iter()
            .filter(|&&config_type| {
                // Skip some exists ConfigCells.
                match config_type {
                    DataType::ConfigCellMain => return self.configs.main.is_none(),
                    DataType::ConfigCellCharSetEmoji
                    | DataType::ConfigCellCharSetDigit
                    | DataType::ConfigCellCharSetEn
                    | DataType::ConfigCellCharSetZhHans
                    | DataType::ConfigCellCharSetZhHant => {
                        if let Some(char_sets) = &self.configs.char_set {
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

        let config_cell_type = config_cell_type();
        let mut config_data_types = Vec::new();
        let mut config_entity_hashes = BTreeMap::new();
        for config_type in unloaded_config_types {
            let args = Bytes::from((config_type.to_owned() as u32).to_le_bytes().to_vec());
            let type_script = config_cell_type.clone().as_builder().args(args.into()).build();
            // There must be one ConfigCell in the cell_deps, no more and no less.
            let ret = util::find_cells_by_script(ScriptType::Type, type_script.as_reader(), Source::CellDep)?;
            assert!(
                ret.len() == 1,
                Error::ConfigCellIsRequired,
                "  Can not find the cell of {:?} in cell_deps. (find_condition: {})",
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
            config_entity_hashes.insert(expected_cell_index, (config_type, expected_entity_hash));

            // Store data type for loading data on demand.
            config_data_types.push(config_type.to_owned())
        }

        debug!("  Load witnesses of the ConfigCells ...");

        macro_rules! assign_config_witness {
            ( $index:expr, $data_type:expr, $property:expr, $witness_type:ty, $entity:expr ) => {
                $property = Some(<$witness_type>::from_slice($entity).map_err(|e| {
                    warn!(
                        "Decoding witnesses[{}](expected to be {:?}) failed: {}",
                        $index,
                        $data_type,
                        e.to_string()
                    );
                    Error::ConfigCellWitnessDecodingError
                })?)
            };
        }

        for (_i, witness_info) in self.witnesses.iter().enumerate() {
            let (index, data_type) = witness_info.to_owned();

            // Skip configs that no need to parse.
            if !config_data_types.contains(&data_type) {
                continue;
            }

            let raw;
            let entity;
            let ret = config_entity_hashes
                .iter()
                .find(|(_, (_data_type, _))| &data_type == *_data_type);
            match ret {
                Some((key, (_, hash))) => {
                    raw = util::load_das_witnesses(index)?;
                    entity = raw
                        .get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..)
                        .ok_or(Error::ConfigCellWitnessDecodingError)?;
                    let entity_hash = util::blake2b_256(entity).to_vec();
                    if &entity_hash == hash {
                        let key_cp = key.to_owned();
                        // debug!("expected: 0x{}", util::hex_string(config_entity_hashes.get(&key).unwrap().as_slice()));
                        config_entity_hashes.remove(&key_cp);
                    } else {
                        // ⚠️ Do not print the whole entity, otherwise memory may be not enough.
                        warn!(
                            "The witness of witness[{}] is corrupted! data_type: {:?} hash: 0x{} entity: {:?}",
                            index,
                            data_type,
                            util::hex_string(entity_hash.as_slice()),
                            entity.get(..40).map(|item| util::hex_string(item) + "...")
                        );
                        return Err(Error::ConfigCellWitnessIsCorrupted);
                    }
                }
                None => continue,
            }

            debug!("  Found matched witness of {:?} at witnesses[{}] .", data_type, index);

            match data_type {
                DataType::ConfigCellAccount => {
                    assign_config_witness!(index, data_type, self.configs.account, ConfigCellAccount, entity)
                }
                DataType::ConfigCellApply => {
                    assign_config_witness!(index, data_type, self.configs.apply, ConfigCellApply, entity)
                }
                DataType::ConfigCellIncome => {
                    assign_config_witness!(index, data_type, self.configs.income, ConfigCellIncome, entity)
                }
                DataType::ConfigCellMain => {
                    assign_config_witness!(index, data_type, self.configs.main, ConfigCellMain, entity)
                }
                DataType::ConfigCellPrice => {
                    assign_config_witness!(index, data_type, self.configs.price, ConfigCellPrice, entity)
                }
                DataType::ConfigCellProposal => {
                    assign_config_witness!(index, data_type, self.configs.proposal, ConfigCellProposal, entity)
                }
                DataType::ConfigCellProfitRate => {
                    assign_config_witness!(index, data_type, self.configs.profit_rate, ConfigCellProfitRate, entity)
                }
                DataType::ConfigCellRelease => {
                    assign_config_witness!(index, data_type, self.configs.release, ConfigCellRelease, entity)
                }
                DataType::ConfigCellSecondaryMarket => {
                    assign_config_witness!(
                        index,
                        data_type,
                        self.configs.secondary_market,
                        ConfigCellSecondaryMarket,
                        entity
                    )
                }
                DataType::ConfigCellSubAccount => {
                    assign_config_witness!(index, data_type, self.configs.sub_account, ConfigCellSubAccount, entity)
                }
                DataType::ConfigCellReverseResolution => {
                    assign_config_witness!(
                        index,
                        data_type,
                        self.configs.reverse_resolution,
                        ConfigCellReverseResolution,
                        entity
                    )
                }
                DataType::ConfigCellRecordKeyNamespace => {
                    self.configs.record_key_namespace = Some(entity.get(WITNESS_LENGTH_BYTES..).unwrap().to_vec());
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
                    self.configs.preserved_account = Some(entity.get(WITNESS_LENGTH_BYTES..).unwrap().to_vec());
                }
                DataType::ConfigCellUnAvailableAccount => {
                    // debug!("length: {}", entity.get(WITNESS_LENGTH_BYTES..).unwrap().len());
                    self.configs.unavailable_account = Some(entity.get(WITNESS_LENGTH_BYTES..).unwrap().to_vec());
                }
                DataType::ConfigCellCharSetEmoji
                | DataType::ConfigCellCharSetDigit
                | DataType::ConfigCellCharSetEn
                | DataType::ConfigCellCharSetZhHans
                | DataType::ConfigCellCharSetZhHant => {
                    let char_set_type = das_types_util::data_type_to_char_set(data_type);
                    let index = char_set_type as usize;
                    let char_set = CharSet {
                        name: char_set_type,
                        // TODO make the meaning of following codes more clear
                        // skip 7 bytes das header, 4 bytes length
                        global: entity.get(WITNESS_LENGTH_BYTES).unwrap() == &1u8,
                        data: entity.get(5..).unwrap().to_vec(),
                    };
                    if self.configs.char_set.is_some() {
                        self.configs
                            .char_set
                            .as_mut()
                            .map(|char_sets| char_sets[index] = Some(char_set));
                    } else {
                        let mut char_sets: Vec<Option<CharSet>> = vec![None; CHAR_SET_LENGTH];
                        char_sets[index] = Some(char_set);
                        self.configs.char_set = Some(char_sets);
                    }
                }
                _ => return Err(Error::ConfigTypeIsUndefined),
            }
        }

        // Check if there is any hash is not used, which means some config is missing.
        assert!(
            config_entity_hashes.is_empty(),
            Error::ConfigIsPartialMissing,
            "Can not find some ConfigCells' witnesses. (can_not_find: {:?})",
            config_entity_hashes
                .iter()
                .map(|(_, (data_type, value))| format!(
                    "data_type: {:?}, entity_hash: 0x{}",
                    data_type,
                    util::hex_string(value.as_slice())
                ))
                .collect::<Vec<String>>()
        );

        Ok(())
    }

    pub fn parse_cell(&mut self) -> Result<(), Error> {
        debug!("Parsing witnesses of all other cells ...");
        // witness format 1: 'das'(3) + DATA_TYPE(4) + molecule

        for (_i, witness) in self.witnesses.iter().enumerate() {
            let (index, data_type) = witness.to_owned();
            // Skip ActionData witness and ConfigCells' witnesses.
            if data_type == DataType::ActionData || self.is_config_data_type(data_type) {
                continue;
            }

            let raw = util::load_das_witnesses(index)?;

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
                    "  Parse witnesses[{}]: {{ data_type: {:?}, source: {:?}, index: {} }}",
                    _i, data_type, source, index
                );
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

    fn get(&self, index: u32, source: Source) -> Result<Option<&(u32, u32, DataType, Vec<u8>, Bytes)>, Error> {
        let group = match source {
            Source::Input => &self.old,
            Source::Output => &self.new,
            Source::CellDep => &self.dep,
            _ => {
                return Err(Error::HardCodedError);
            }
        };

        Ok(group.iter().find(|&(i, _, _, _, _)| *i == index))
    }

    pub fn verify_hash(&self, index: usize, source: Source) -> Result<(), Error> {
        let data = util::load_cell_data(index, source)?;
        let expected_hash = match data.get(..32) {
            Some(bytes) => bytes,
            _ => {
                warn!("  {:?}[{}] Can not get entity hash from outputs_data.", source, index);
                return Err(Error::InvalidCellData);
            }
        };

        if let Some((_, _, _, _hash, _)) = self.get(index as u32, source)? {
            assert!(
                expected_hash == _hash,
                Error::WitnessDataHashOrTypeMissMatch,
                "{:?}[{}] Can not find witness.(expected_hash: 0x{}, current_hash: 0x{})",
                source,
                index,
                util::hex_string(expected_hash),
                util::hex_string(_hash)
            );
        } else {
            // This error means the there is no witness.data.dep/old/new.index matches the index of the cell.
            warn!(
                "  {:?}[{}] Can not find witness.(expected_hash: 0x{})",
                source,
                index,
                util::hex_string(expected_hash)
            );
            return Err(Error::WitnessDataIndexMissMatch);
        }

        Ok(())
    }

    pub fn verify_and_get(
        &self,
        data_type: DataType,
        index: usize,
        source: Source,
    ) -> Result<(u32, DataType, &Bytes), Error> {
        let data = util::load_cell_data(index, source)?;
        let hash = match data.get(..32) {
            Some(bytes) => bytes,
            _ => {
                warn!("  {:?}[{}] Can not get entity hash from outputs_data.", source, index);
                return Err(Error::InvalidCellData);
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
    ) -> Result<(u32, DataType, &Bytes), Error> {
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
                return Err(Error::WitnessDataHashOrTypeMissMatch);
            }
        } else {
            // This error means the there is no witness.data.dep/old/new.index matches the index of the cell.
            warn!(
                "Can not find witness at {:?}[{}], expected hash: 0x{}",
                source,
                index,
                util::hex_string(expected_hash)
            );
            return Err(Error::WitnessDataIndexMissMatch);
        }

        Ok((version, data_type, entity))
    }

    fn is_config_data_type(&self, data_type: DataType) -> bool {
        let data_type_in_int = data_type as u32;
        data_type_in_int >= 100 && data_type_in_int <= 199999
    }
}
