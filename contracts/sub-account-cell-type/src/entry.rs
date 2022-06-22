use alloc::{borrow::ToOwned, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, dynamic_loading_c_impl::CKBDLContext, high_level};
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
use das_dynamic_libs::{
    constants::{DymLibSize, ETH_LIB_CODE_HASH, TRON_LIB_CODE_HASH},
    sign_lib::{SignLib, SignLibWith2Methods},
};
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
        b"recycle_expired_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"config_sub_account_custom_script" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_account_cells, output_account_cells) = util::find_cells_by_type_id_in_inputs_and_outputs(
                ScriptType::Type,
                config_main.type_id_table().account_cell(),
            )?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[0],
                &output_account_cells,
                &[0],
            )?;

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            verifiers::misc::verify_no_more_cells_with_same_lock(
                sender_lock.as_reader(),
                &input_account_cells,
                Source::Input,
            )?;

            let input_account_cell_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_cell_reader = input_account_cell_witness.as_reader();
            let output_account_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_cell_reader = output_account_cell_witness.as_reader();

            verifiers::account_cell::verify_status(
                &input_account_cell_reader,
                AccountStatus::Normal,
                input_account_cells[0],
                Source::Input,
            )?;

            verifiers::account_cell::verify_account_expiration(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
            )?;

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;

            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_account_cell_reader,
                &output_account_cell_reader,
                None,
                vec![],
                vec![],
            )?;

            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "SubAccountCell",
                &input_sub_account_cells,
                &[1],
                &output_sub_account_cells,
                &[1],
            )?;

            verifiers::sub_account_cell::verify_sub_account_parent_id(
                input_sub_account_cells[0],
                Source::Input,
                input_account_cell_reader.id().raw_data(),
            )?;

            let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

            verify_sub_account_transaction_fee(
                config_sub_account,
                action,
                input_sub_account_capacity,
                &input_sub_account_data,
                output_sub_account_capacity,
                &output_sub_account_data,
            )?;

            let input_sub_account_custom_script =
                data_parser::sub_account_cell::get_custom_script(&input_sub_account_data);
            let output_sub_account_custom_script =
                data_parser::sub_account_cell::get_custom_script(&output_sub_account_data);
            // manual::verify_custom_script_changed
            assert!(
                input_sub_account_custom_script != output_sub_account_custom_script,
                Error::SubAccountCustomScriptError,
                "outputs[{}] The custom script of SubAccountCell should be different in inputs and outputs.",
                output_sub_account_cells[0]
            );

            verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["custom_script"],
            )?;
        }
        b"create_sub_account" | b"edit_sub_account" | b"renew_sub_account" | b"recycle_sub_account" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let mut eth_lib = unsafe { CKBDLContext::<DymLibSize>::new() };
            let mut tron_lib = unsafe { CKBDLContext::<DymLibSize>::new() };
            let mut eth = None;
            let mut tron = None;

            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

            verify_sub_account_capacity_is_enough(
                config_sub_account,
                input_sub_account_cells[0],
                input_sub_account_capacity,
                &input_sub_account_data,
                output_sub_account_cells[0],
                output_sub_account_capacity,
                &output_sub_account_data,
            )?;

            verify_sub_account_transaction_fee(
                config_sub_account,
                action,
                input_sub_account_capacity,
                &input_sub_account_data,
                output_sub_account_capacity,
                &output_sub_account_data,
            )?;

            let mut parent_account;
            match action {
                b"create_sub_account" => {
                    let (input_account_cells, output_account_cells) =
                        util::find_cells_by_type_id_in_inputs_and_outputs(
                            ScriptType::Type,
                            config_main.type_id_table().account_cell(),
                        )?;
                    verifiers::common::verify_cell_number_and_position(
                        "AccountCell",
                        &input_account_cells,
                        &[0],
                        &output_account_cells,
                        &[0],
                    )?;

                    let input_account_cell_witness =
                        util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
                    let input_account_cell_reader = input_account_cell_witness.as_reader();
                    let output_account_cell_witness =
                        util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
                    let output_account_cell_reader = output_account_cell_witness.as_reader();

                    verifiers::account_cell::verify_status(
                        &input_account_cell_reader,
                        AccountStatus::Normal,
                        input_account_cells[0],
                        Source::Input,
                    )?;

                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        Source::Input,
                        timestamp,
                    )?;

                    verifiers::account_cell::verify_sub_account_enabled(
                        &input_account_cell_reader,
                        input_account_cells[0],
                        Source::Input,
                    )?;

                    verifiers::account_cell::verify_account_capacity_not_decrease(
                        input_account_cells[0],
                        output_account_cells[0],
                    )?;

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

                    verifiers::common::verify_cell_number_and_position(
                        "SubAccountCell",
                        &input_sub_account_cells,
                        &[1],
                        &output_sub_account_cells,
                        &[1],
                    )?;

                    verify_sub_account_cell_is_consistent(
                        input_sub_account_cells[0],
                        output_sub_account_cells[0],
                        vec!["profit"],
                    )?;
                }
                b"edit_sub_account" => {
                    let dep_account_cells = util::find_cells_by_type_id(
                        ScriptType::Type,
                        config_main.type_id_table().account_cell(),
                        Source::CellDep,
                    )?;
                    verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

                    let dep_account_cell_witness =
                        util::parse_account_cell_witness(&parser, dep_account_cells[0], Source::CellDep)?;
                    let dep_account_cell_reader = dep_account_cell_witness.as_reader();

                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        dep_account_cells[0],
                        Source::CellDep,
                        timestamp,
                    )?;

                    verifiers::account_cell::verify_sub_account_enabled(
                        &dep_account_cell_reader,
                        dep_account_cells[0],
                        Source::CellDep,
                    )?;

                    verifiers::common::verify_cell_number_and_position(
                        "SubAccountCell",
                        &input_sub_account_cells,
                        &[0],
                        &output_sub_account_cells,
                        &[0],
                    )?;

                    parent_account = dep_account_cell_reader.account().as_readable();
                    parent_account.extend(ACCOUNT_SUFFIX.as_bytes());

                    verify_sub_account_cell_is_consistent(
                        input_sub_account_cells[0],
                        output_sub_account_cells[0],
                        vec!["smt_root"],
                    )?;

                    if cfg!(not(feature = "dev")) {
                        // CAREFUL Proof verification has been skipped in development mode.
                        // TODO Refactor the temporary solution of dynamic library loading ...
                        let lib = eth_lib
                            .load(&ETH_LIB_CODE_HASH)
                            .expect("The shared lib should be loaded successfully.");
                        eth = Some(SignLibWith2Methods {
                            c_validate: unsafe {
                                lib.get(b"validate")
                                    .expect("Load function 'validate' from library failed.")
                            },
                            c_validate_str: unsafe {
                                lib.get(b"validate_str")
                                    .expect("Load function 'validate_str' from library failed.")
                            },
                        });

                        let lib = tron_lib
                            .load(&TRON_LIB_CODE_HASH)
                            .expect("The shared lib should be loaded successfully.");
                        tron = Some(SignLibWith2Methods {
                            c_validate: unsafe {
                                lib.get(b"validate")
                                    .expect("Load function 'validate' from library failed.")
                            },
                            c_validate_str: unsafe {
                                lib.get(b"validate_str")
                                    .expect("Load function 'validate_str' from library failed.")
                            },
                        });
                    }
                }
                b"renew_sub_account" => todo!(),
                b"recycle_sub_account" => todo!(),
                _ => unreachable!(),
            }

            let sign_lib = SignLib::new(eth, tron, None);

            debug!("Start iterating sub-account witnesses ...");

            let mut first_root = &vec![];
            let mut last_root = &vec![];
            let sub_account_parser = SubAccountWitnessesParser::new()?;
            let mut profit_to_das = 0;
            for (i, witness_ret) in sub_account_parser.iter().enumerate() {
                match witness_ret {
                    Ok(witness) => {
                        // Store the first SMT root in the transaction, and verify it later.
                        if first_root.is_empty() {
                            first_root = &witness.prev_root;
                        }

                        debug!(
                            "witnesses[{}] Verify if the root of witnesses[{}] and witnesses[{}] is sequential.",
                            witness.index,
                            witness.index,
                            witness.index + 1
                        );

                        match sub_account_parser.get(i + 1) {
                            Some(Ok(next_witness)) => {
                                let current_root = &witness.current_root;
                                let prev_root_of_next = &next_witness.prev_root;

                                assert!(
                                    current_root == prev_root_of_next,
                                    Error::SubAccountCellSMTRootError,
                                    "witnesses[{}] The roots in sub-account witnesses should be sequential, but witnesses[{}] and witnesses[{}] is not.",
                                    witness.index,
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
                                verifiers::sub_account_cell::verify_suffix_with_parent_account(
                                    witness.index,
                                    sub_account_reader,
                                    &parent_account,
                                )?;

                                smt_verify_sub_account_is_creatable(witness)?;

                                debug!("witnesses[{}] Verify if the account is registrable.", witness.index);

                                let account_chars = sub_account_reader.account();
                                verifiers::account_cell::verify_account_chars(&parser, account_chars)?;
                                verifiers::account_cell::verify_account_chars_max_length(&parser, account_chars)?;

                                verifiers::sub_account_cell::verify_initial_properties(
                                    witness.index,
                                    sub_account_reader,
                                    timestamp,
                                )?;

                                debug!("Sum profit base on registered years in all sub-accounts.");

                                let expired_at = u64::from(sub_account_reader.expired_at());
                                let registered_at = u64::from(sub_account_reader.registered_at());
                                let expiration_years = (expired_at - registered_at) / YEAR_SEC;
                                profit_to_das +=
                                    u64::from(config_sub_account.new_sub_account_price()) * expiration_years;
                            }
                            b"edit_sub_account" => {
                                verifiers::sub_account_cell::verify_suffix_with_parent_account(
                                    witness.index,
                                    sub_account_reader,
                                    &parent_account,
                                )?;

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

                                verifiers::sub_account_cell::verify_unlock_role(witness)?;
                                verifiers::sub_account_cell::verify_sub_account_sig(witness, &sign_lib)?;
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
                                    // manual::verify_expired_at_not_editable
                                    SubAccountEditValue::ExpiredAt(_) => {
                                        warn!("witnesses[{}] Can not edit witness.sub_account.expired_at in this transaction.", witness.index);
                                        return Err(Error::SubAccountFieldNotEditable);
                                    }
                                    // manual::verify_edit_value_not_empty
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

            verify_sub_account_cell_smt_root(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                first_root,
                last_root,
            )?;

            match action {
                b"create_sub_account" => {
                    verify_profit_to_das(
                        action,
                        output_sub_account_cells[0],
                        &input_sub_account_data,
                        &output_sub_account_data,
                        profit_to_das,
                    )?;
                }
                _ => {}
            }
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn verify_sub_account_capacity_is_enough(
    config: ConfigCellSubAccountReader,
    input_index: usize,
    input_capacity: u64,
    input_data: &[u8],
    output_index: usize,
    output_capacity: u64,
    output_data: &[u8],
) -> Result<(), Error> {
    let basic_capacity = u64::from(config.basic_capacity());
    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();

    assert!(
        input_capacity >= input_das_profit + input_owner_profit + basic_capacity,
        Error::SubAccountCellCapacityError,
        "inputs[{}] The capacity of SubAccountCell should contains profit and basic_capacity, but its not enough.(capacity: {}, das_profit: {}, owner_profit: {})",
        input_index,
        input_capacity,
        input_das_profit,
        input_owner_profit
    );
    assert!(
        output_capacity >= output_das_profit + output_owner_profit + basic_capacity,
        Error::SubAccountCellCapacityError,
        "outputs[{}] The capacity of SubAccountCell should contains profit and basic_capacity, but its not enough.(capacity: {}, das_profit: {}, owner_profit: {})",
        output_index,
        output_capacity,
        output_das_profit,
        output_owner_profit
    );

    Ok(())
}

fn verify_sub_account_transaction_fee(
    config: ConfigCellSubAccountReader,
    action: &[u8],
    input_capacity: u64,
    input_data: &[u8],
    output_capacity: u64,
    output_data: &[u8],
) -> Result<(), Error> {
    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();

    let fee = match action {
        b"create_sub_account" => u64::from(config.create_fee()),
        b"edit_sub_account" => u64::from(config.edit_fee()),
        b"renew_sub_account" => u64::from(config.renew_fee()),
        b"recycle_sub_account" => u64::from(config.recycle_fee()),
        _ => u64::from(config.common_fee()),
    };
    let basic_capacity = u64::from(config.basic_capacity());
    let input_remain_fees = input_capacity - input_das_profit - input_owner_profit - basic_capacity;
    let output_remain_fees = output_capacity - output_das_profit - output_owner_profit - basic_capacity;

    assert!(
        input_remain_fees <= fee + output_remain_fees,
        Error::SubAccountCellCapacityError,
        "The transaction fee should be equal to or less than {} .(output_remain_fees: {} = output_capacity - output_profit - basic_capacity, input_remain_fees: {} = ...)",
        fee,
        output_remain_fees,
        input_remain_fees
    );

    Ok(())
}

fn verify_sub_account_cell_is_consistent(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
    except: Vec<&str>,
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

    let input_sub_account_data = high_level::load_cell_data(input_sub_account_cell, Source::Input)?;
    let output_sub_account_data = high_level::load_cell_data(output_sub_account_cell, Source::Output)?;

    macro_rules! assert_field_consistent_if_not_except {
        ($field_name:expr, $get_name:ident) => {
            if !except.contains(&$field_name) {
                let input_value = data_parser::sub_account_cell::$get_name(&input_sub_account_data).unwrap();
                let output_value = data_parser::sub_account_cell::$get_name(&output_sub_account_data).unwrap();
                assert!(
                    input_value == output_value,
                    Error::SubAccountCellConsistencyError,
                    "The SubAccountCell.data.{} should be consistent in inputs and outputs.",
                    $field_name
                );
            }
        };
    }

    assert_field_consistent_if_not_except!("smt_root", get_smt_root);
    assert_field_consistent_if_not_except!("das_profit", get_das_profit);
    assert_field_consistent_if_not_except!("owner_profit", get_owner_profit);
    assert_field_consistent_if_not_except!("custom_script", get_custom_script);

    Ok(())
}

fn verify_sub_account_cell_smt_root(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
    first_root_in_witnesses: &[u8],
    last_root_in_witnesses: &[u8],
) -> Result<(), Error> {
    debug!("Verify if the first SMT root in sub-account witnesses is equal to the SubAccountCell.data in inputs.");

    let data = high_level::load_cell_data(input_sub_account_cell, Source::Input)?;
    let first_root = data_parser::sub_account_cell::get_smt_root(&data);

    assert!(
        first_root == Some(first_root_in_witnesses),
        Error::SubAccountWitnessSMTRootError,
        "The first SMT root in sub-account witnesses should be equal to the SubAccountCell.data in inputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(first_root.or(Some(&[])).unwrap()),
        util::hex_string(first_root_in_witnesses)
    );

    let data = high_level::load_cell_data(output_sub_account_cell, Source::Output)?;
    let last_root = data_parser::sub_account_cell::get_smt_root(&data);

    assert!(
        last_root == Some(last_root_in_witnesses),
        Error::SubAccountWitnessSMTRootError,
        "The last SMT root in sub-account witnesses should be equal to the SubAccountCell.data in outputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(last_root.or(Some(&[])).unwrap()),
        util::hex_string(last_root_in_witnesses)
    );

    Ok(())
}

fn verify_profit_to_das(
    action: &[u8],
    cell_index: usize,
    input_profit: u64,
    output_profit: u64,
    profit_to_das: u64,
) -> Result<(), Error> {
    if action == b"create_sub_account" {
        assert!(
            output_profit == input_profit + profit_to_das,
            Error::SubAccountProfitError,
            "outputs[{}] The profit of SubAccountCell should contains the new register fees. (output_profit: {}, input_profit: {}, expected_register_fee: {})",
            cell_index,
            output_profit,
            input_profit,
            profit_to_das
        );
    } else {
        // TODO Implement withdraw action
        todo!();
    }

    Ok(())
}

fn gen_smt_key_by_account_id(account_id: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let key_pre = [account_id, &[0u8; 12]].concat();
    key.copy_from_slice(&key_pre);
    key
}

fn smt_verify_sub_account_is_creatable(witness: &SubAccountWitness) -> Result<(), Error> {
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!(
        "witnesses[{}] Verify if the sub-account was not exist in the SMT before.(key: 0x{})",
        witness.index,
        util::hex_string(&key)
    );
    let prev_root = witness.prev_root.as_slice();
    let zero_val = [0u8; 32];
    verifiers::sub_account_cell::verify_smt_proof(key, zero_val, prev_root.try_into().unwrap(), proof)?;

    debug!(
        "witnesses[{}] Verify if the sub-account is in the SMT now.",
        witness.index
    );
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

    debug!(
        "witnesses[{}] Verify if the current state of the sub-account was in the SMT before.(key: 0x{})",
        witness.index,
        util::hex_string(&key)
    );
    let prev_root = witness.prev_root.as_slice();
    let prev_val: [u8; 32] = blake2b_256(witness.sub_account.as_slice()).to_vec().try_into().unwrap();
    // debug!("prev_val = 0x{}", util::hex_string(&prev_val));
    // debug!("prev_val_raw = 0x{}", util::hex_string(witness.sub_account.as_slice()));
    // debug!("prev_val_prettier = {}", witness.sub_account.as_prettier());
    verifiers::sub_account_cell::verify_smt_proof(key, prev_val, prev_root.try_into().unwrap(), proof)?;

    debug!(
        "witnesses[{}] Verify if the new state of the sub-account is in the SMT now.",
        witness.index
    );
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
