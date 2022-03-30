use alloc::{borrow::ToOwned, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, high_level};
use core::{convert::TryInto, result::Result};
use das_core::{
    assert,
    constants::*,
    data_parser, debug,
    error::Error,
    sub_account_witness_parser::{SubAccountEditValue, SubAccountWitness, SubAccountWitnessesParser},
    util::{self, blake2b_256},
    verifiers, warn,
    witness_parser::WitnessesParser,
};
use das_map::map::Map;
use das_types::{
    constants::AccountStatus,
    packed::*,
    prelude::{Builder, Entity},
    prettier::Prettier,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running sub-account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );

    match action {
        b"enable_sub_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"create_sub_account" | b"edit_sub_account" | b"renew_sub_account" | b"recycle_sub_account" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;
            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verify_transaction_fee_spent_correctly(
                action,
                config_sub_account,
                input_sub_account_cells[0],
                output_sub_account_cells[0],
            )?;

            let mut parent_account = Vec::new();
            match action {
                b"create_sub_account" => {
                    let (input_account_cells, output_account_cells) =
                        util::find_cells_by_type_id_in_inputs_and_outputs(
                            ScriptType::Type,
                            config_main.type_id_table().account_cell(),
                        )?;
                    let input_account_cell_witness =
                        util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
                    let input_account_cell_reader = input_account_cell_witness.as_reader();
                    let output_account_cell_witness =
                        util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
                    let output_account_cell_reader = output_account_cell_witness.as_reader();

                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_account_cell_reader,
                        &output_account_cell_reader,
                        None,
                        vec![],
                        vec![],
                    )?;

                    parent_account = output_account_cell_reader.account().as_readable();
                    parent_account.extend(ACCOUNT_SUFFIX.as_bytes());
                }
                b"edit_sub_account" => {
                    // Nothing to do here
                }
                b"renew_sub_account" => todo!(),
                b"recycle_sub_account" => todo!(),
                _ => unreachable!(),
            }

            debug!("Start iterating sub-account witnesses ...");

            let mut first_root = &vec![];
            let mut last_root = &vec![];
            let sub_account_parser = SubAccountWitnessesParser::new()?;
            let mut expected_register_fee = 0;
            for witness_ret in sub_account_parser.iter() {
                match witness_ret {
                    Ok(witness) => {
                        // Store the first SMT root in the transaction, and verify it later.
                        if first_root.is_empty() {
                            first_root = &witness.prev_root;
                        }

                        debug!(
                            "Verify if the root of witnesses[{}] and witnesses[{}] is sequential.",
                            witness.index,
                            witness.index + 1
                        );

                        match sub_account_parser.get(witness.index + 1) {
                            Some(Ok(next_witness)) => {
                                let current_root = &witness.current_root;
                                let prev_root_of_next = &next_witness.prev_root;

                                assert!(
                                    current_root == prev_root_of_next,
                                    Error::SubAccountCellSMTRootError,
                                    "The roots in sub-account witnesses should be sequential, but witnesses[{}] and witnesses[{}] is not.",
                                    witness.index,
                                    next_witness.index
                                );
                            }
                            Some(Err(err)) => return Err(err),
                            None => {
                                // For the last sub-account witness, there will be no next.
                                // Store the last SMT root in the transaction, and verify it later.
                                last_root = &witness.current_root;
                            }
                        }

                        let sub_account_reader = witness.sub_account.as_reader();
                        match action {
                            b"create_sub_account" => {
                                smt_verify_sub_account_is_creatable(witness)?;

                                debug!("Verify if the account is registrable.");

                                let account_chars = sub_account_reader.account();
                                verifiers::account_cell::verify_account_chars(&parser, account_chars)?;
                                verifiers::account_cell::verify_account_chars_max_length(&parser, account_chars)?;

                                debug!("Verify if the initial values of sub-account's fields is filled properly.");

                                verifiers::sub_account_cell::verify_initial_lock(witness.index, sub_account_reader)?;
                                verifiers::sub_account_cell::verify_initial_id(witness.index, sub_account_reader)?;
                                verifiers::sub_account_cell::verify_initial_registered_at(
                                    witness.index,
                                    sub_account_reader,
                                    timestamp,
                                )?;
                                verifiers::sub_account_cell::verify_suffix_with_parent_account(
                                    witness.index,
                                    sub_account_reader,
                                    &parent_account,
                                )?;
                                verifiers::sub_account_cell::verify_status(
                                    witness.index,
                                    sub_account_reader,
                                    AccountStatus::Normal,
                                )?;

                                assert!(
                                    sub_account_reader.records().len() == 0,
                                    Error::AccountCellRecordNotEmpty,
                                    "witnesses[{}] The witness.sub_account.records of {} should be empty.",
                                    witness.index,
                                    util::get_sub_account_name_from_reader(sub_account_reader)
                                );

                                let enable_sub_account = u8::from(sub_account_reader.enable_sub_account());
                                assert!(
                                    enable_sub_account == 0,
                                    Error::SubAccountInitialValueError,
                                    "witnesses[{}] The witness.sub_account.enable_sub_account of {} should be 0 .",
                                    witness.index,
                                    util::get_sub_account_name_from_reader(sub_account_reader)
                                );

                                let renew_sub_account_price = u64::from(sub_account_reader.renew_sub_account_price());
                                assert!(
                                    renew_sub_account_price == 0,
                                    Error::SubAccountInitialValueError,
                                    "witnesses[{}] The witness.sub_account.renew_sub_account_price of {} should be 0 .",
                                    witness.index,
                                    util::get_sub_account_name_from_reader(sub_account_reader)
                                );

                                let nonce = u64::from(sub_account_reader.nonce());
                                assert!(
                                    nonce == 0,
                                    Error::SubAccountInitialValueError,
                                    "witnesses[{}] The witness.sub_account.nonce of {} should be 0 .",
                                    witness.index,
                                    util::get_sub_account_name_from_reader(sub_account_reader)
                                );

                                debug!("Verify and count witness.sub_account.expired_at in every sub-account.");

                                let expired_at = u64::from(sub_account_reader.expired_at());
                                assert!(
                                    expired_at >= timestamp + YEAR_SEC,
                                    Error::SubAccountInitialValueError,
                                    "witnesses[{}] The witness.sub_account.expired_at should be at least one year.(expected: >= {}, current: {})",
                                    witness.index,
                                    timestamp + YEAR_SEC,
                                    expired_at
                                );

                                let registered_at = u64::from(sub_account_reader.registered_at());
                                let expiration_years = (expired_at - registered_at) / YEAR_SEC;
                                expected_register_fee +=
                                    u64::from(config_sub_account.new_sub_account_price()) * expiration_years;
                            }
                            b"edit_sub_account" => {
                                let new_sub_account = generate_new_sub_account_by_edit_value(
                                    witness.sub_account.clone(),
                                    &witness.edit_value,
                                )?;
                                let new_sub_account_reader = new_sub_account.as_reader();

                                debug!(
                                    "witnesses[{}] Calculated new sub-account structure is: {}",
                                    witness.index,
                                    new_sub_account_reader.as_prettier()
                                );

                                smt_verify_sub_account_is_editable(witness, new_sub_account_reader)?;

                                verifiers::sub_account_cell::verify_sub_account_sig(
                                    witness.edit_key.as_slice(),
                                    witness.edit_value_bytes.as_slice(),
                                    witness.sub_account.nonce().as_slice(),
                                    witness.signature.as_slice(),
                                    witness.sign_args.as_slice(),
                                )?;

                                verifiers::sub_account_cell::verify_expiration(
                                    config_account,
                                    witness.index,
                                    sub_account_reader,
                                    timestamp,
                                )?;
                                verifiers::sub_account_cell::verify_status(
                                    witness.index,
                                    sub_account_reader,
                                    AccountStatus::Normal,
                                )?;

                                match &witness.edit_value {
                                    SubAccountEditValue::Owner(new_args) | SubAccountEditValue::Manager(new_args) => {
                                        let sub_account_reader = witness.sub_account.as_reader();
                                        let current_args = sub_account_reader.lock().args().raw_data();
                                        let (
                                            current_owner_type,
                                            current_owner_args,
                                            current_manager_type,
                                            current_manager_args,
                                        ) = data_parser::das_lock_args::get_owner_and_manager(current_args)?;
                                        let (new_owner_type, new_owner_args, new_manager_type, new_manager_args) =
                                            data_parser::das_lock_args::get_owner_and_manager(new_args)?;

                                        if let SubAccountEditValue::Owner(_) = &witness.edit_value {
                                            debug!(
                                                "witnesses[{}] Verify if owner has been changed correctly.",
                                                witness.index
                                            );

                                            assert!(
                                                current_owner_type != new_owner_type
                                                    || current_owner_args != new_owner_args,
                                                Error::SubAccountEditLockError,
                                                "witnesses[{}] The owner fields in args should be consistent.",
                                                witness.index
                                            );

                                            // Skip verifying manger, because owner has been changed.
                                        } else {
                                            debug!(
                                                "witnesses[{}] Verify if manager has been changed correctly.",
                                                witness.index
                                            );

                                            assert!(
                                                current_owner_type == new_owner_type
                                                    && current_owner_args == new_owner_args,
                                                Error::SubAccountEditLockError,
                                                "witnesses[{}] The owner fields in args should be consistent.",
                                                witness.index
                                            );

                                            assert!(
                                                current_manager_type != new_manager_type
                                                    || current_manager_args != new_manager_args,
                                                Error::SubAccountEditLockError,
                                                "witnesses[{}] The manager fields in args should be changed.",
                                                witness.index
                                            );
                                        }
                                    }
                                    SubAccountEditValue::Records(records) => {
                                        verifiers::account_cell::verify_records_keys(&parser, records.as_reader())?;
                                    }
                                    SubAccountEditValue::ExpiredAt(_) => {
                                        warn!("witnesses[{}] Can not edit witness.sub_account.expired_at in this transaction.", witness.index);
                                        return Err(Error::SubAccountFieldNotEditable);
                                    }
                                    SubAccountEditValue::None => {
                                        warn!(
                                            "witnesses[{}] The witness.edit_value should not be empty.",
                                            witness.index
                                        );
                                        return Err(Error::SubAccountFieldNotEditable);
                                    }
                                }
                            }
                            b"renew_sub_account" => todo!(),
                            b"recycle_sub_account" => todo!(),
                            _ => unreachable!(),
                        }
                    }
                    Err(err) => return Err(err),
                }
            }

            verify_sub_account_cell_is_consistent(input_sub_account_cells[0], output_sub_account_cells[0])?;
            verify_sub_account_cell_smt_root(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                first_root,
                last_root,
            )?;

            if action == b"create_sub_account" {
                let das_wallet_lock = Script::from(das_wallet_lock());
                let mut profit_map = Map::new();
                profit_map.insert(das_wallet_lock.as_reader().as_slice().to_vec(), expected_register_fee);

                verifiers::income_cell::verify_income_cells(&parser, profit_map)?;
            }
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn verify_transaction_fee_spent_correctly(
    action: &[u8],
    config: ConfigCellSubAccountReader,
    input_sub_account: usize,
    output_sub_account: usize,
) -> Result<(), Error> {
    debug!("Check if the fee in the AccountCell is spent correctly.");

    let storage_capacity = u64::from(config.basic_capacity());
    let fee = match action {
        b"create_sub_account" => u64::from(config.create_fee()),
        b"edit_sub_account" => u64::from(config.edit_fee()),
        b"renew_sub_account" => u64::from(config.renew_fee()),
        b"recycle_sub_account" => u64::from(config.recycle_fee()),
        _ => u64::from(config.common_fee()),
    };

    verifiers::common::verify_tx_fee_spent_correctly(
        "SubAccountCell",
        input_sub_account,
        output_sub_account,
        fee,
        storage_capacity,
    )?;

    Ok(())
}

fn verify_sub_account_cell_is_consistent(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
) -> Result<(), Error> {
    debug!("Verify if the SubAccountCell is consistent in inputs and outputs.");

    let input_sub_account_cell_lock = high_level::load_cell_lock(input_sub_account_cell, Source::Input)?;
    let output_sub_account_cell_lock = high_level::load_cell_lock(output_sub_account_cell, Source::Output)?;

    assert!(
        util::is_entity_eq(&input_sub_account_cell_lock, &output_sub_account_cell_lock),
        Error::SubAccountCellConsistencyError,
        "The SubAccountCell.lock should be consistent in inputs and outputs."
    );

    let input_sub_account_cell_type =
        high_level::load_cell_type(input_sub_account_cell, Source::Input)?.expect("The type script should exist.");
    let output_sub_account_cell_type =
        high_level::load_cell_type(output_sub_account_cell, Source::Output)?.expect("The type script should exist.");

    assert!(
        util::is_entity_eq(&input_sub_account_cell_type, &output_sub_account_cell_type),
        Error::SubAccountCellConsistencyError,
        "The SubAccountCell.type should be consistent in inputs and outputs."
    );

    Ok(())
}

fn verify_sub_account_cell_smt_root(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
    first_root_in_witnesses: &[u8],
    last_root_in_witnesses: &[u8],
) -> Result<(), Error> {
    debug!("Verify if the first SMT root in sub-account witnesses is equal to the SubAccountCell.data in inputs.");

    let first_root = high_level::load_cell_data(input_sub_account_cell, Source::Input)?;

    assert!(
        &first_root == first_root_in_witnesses,
        Error::SubAccountWitnessSMTRootError,
        "The first SMT root in sub-account witnesses should be equal to the SubAccountCell.data in inputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(&first_root),
        util::hex_string(first_root_in_witnesses)
    );

    let last_root = high_level::load_cell_data(output_sub_account_cell, Source::Output)?;

    assert!(
        &last_root == last_root_in_witnesses,
        Error::SubAccountWitnessSMTRootError,
        "The last SMT root in sub-account witnesses should be equal to the SubAccountCell.data in outputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(&last_root),
        util::hex_string(last_root_in_witnesses)
    );

    Ok(())
}

fn gen_smt_key_by_account_id(account_id: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let key_pre = [account_id, &[0u8; 12]].concat();
    key.copy_from_slice(&key_pre);
    debug!("gen_smt_key_by_account_id, key: {}", util::hex_string(&key));
    key
}

fn smt_verify_sub_account_is_creatable(witness: &SubAccountWitness) -> Result<(), Error> {
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!("Verify if the sub-account was not exist in the SMT before.");
    let prev_root = witness.prev_root.as_slice();
    let zero_val = [0u8; 32];
    verifiers::sub_account_cell::verify_smt_proof(key, zero_val, prev_root.try_into().unwrap(), proof)?;

    debug!("Verify if the sub-account is in the SMT now.");
    let current_root = witness.current_root.as_slice();
    let current_val = blake2b_256(witness.sub_account.as_slice()).to_vec().try_into().unwrap();
    verifiers::sub_account_cell::verify_smt_proof(key, current_val, current_root.try_into().unwrap(), proof)?;

    Ok(())
}

fn smt_verify_sub_account_is_editable(
    witness: &SubAccountWitness,
    new_sub_account: SubAccountReader,
) -> Result<(), Error> {
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!("Verify if the current state of the sub-account was in the SMT before.");
    let prev_root = witness.prev_root.as_slice();
    let prev_val: [u8; 32] = blake2b_256(witness.sub_account.as_slice()).to_vec().try_into().unwrap();
    // debug!("prev_val = 0x{}", util::hex_string(&prev_val));
    // debug!("prev_val_raw = 0x{}", util::hex_string(witness.sub_account.as_slice()));
    // debug!("prev_val_prettier = {}", witness.sub_account.as_prettier());
    verifiers::sub_account_cell::verify_smt_proof(key, prev_val, prev_root.try_into().unwrap(), proof)?;

    debug!("Verify if the new state of the sub-account is in the SMT now.");
    let current_root = witness.current_root.as_slice();
    let current_val: [u8; 32] = blake2b_256(new_sub_account.as_slice()).to_vec().try_into().unwrap();
    // debug!("current_val = 0x{}", util::hex_string(&current_val));
    // debug!("current_val_raw = 0x{}", util::hex_string(new_sub_account.as_slice()));
    // debug!("current_val_prettier = {}", new_sub_account.as_prettier());
    verifiers::sub_account_cell::verify_smt_proof(key, current_val, current_root.try_into().unwrap(), proof)?;

    Ok(())
}

fn generate_new_sub_account_by_edit_value(
    sub_account: SubAccount,
    edit_value: &SubAccountEditValue,
) -> Result<SubAccount, Error> {
    let current_nonce = u64::from(sub_account.nonce());

    let mut sub_account_builder = match edit_value {
        SubAccountEditValue::ExpiredAt(val) => {
            let sub_account_builder = sub_account.as_builder();
            sub_account_builder.expired_at(val.to_owned())
        }
        SubAccountEditValue::Owner(val) | SubAccountEditValue::Manager(val) => {
            let mut lock_builder = sub_account.lock().as_builder();
            let mut sub_account_builder = sub_account.as_builder();

            // Verify if the edit_value is a valid format.
            data_parser::das_lock_args::get_owner_and_manager(val)?;

            lock_builder = lock_builder.args(Bytes::from(val.to_owned()));
            sub_account_builder = sub_account_builder.lock(lock_builder.build());

            if let SubAccountEditValue::Owner(_) = edit_value {
                sub_account_builder = sub_account_builder.records(Records::default())
            }

            sub_account_builder
        }
        SubAccountEditValue::Records(val) => {
            let sub_account_builder = sub_account.as_builder();
            sub_account_builder.records(val.to_owned())
        }
        _ => unreachable!(),
    };

    // Every time a sub-account is edited, its nonce must  increase by 1 .
    sub_account_builder = sub_account_builder.nonce(Uint64::from(current_nonce + 1));

    Ok(sub_account_builder.build())
}
