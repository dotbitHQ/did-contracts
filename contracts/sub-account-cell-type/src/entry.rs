use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, cstr_core::CStr, dynamic_loading_c_impl::CKBDLContext, error::SysError, high_level};
use core::{convert::TryInto, result::Result};
use das_core::{
    assert as das_assert,
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
            let input_sub_account_script_args =
                data_parser::sub_account_cell::get_custom_script_args(&input_sub_account_data);
            let output_sub_account_script_args =
                data_parser::sub_account_cell::get_custom_script_args(&output_sub_account_data);
            // manual::verify_custom_script_changed
            das_assert!(
                input_sub_account_custom_script != output_sub_account_custom_script
                    || input_sub_account_script_args != output_sub_account_script_args,
                Error::SubAccountCustomScriptError,
                "outputs[{}] The custom script of SubAccountCell should be different in inputs and outputs.",
                output_sub_account_cells[0]
            );

            verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["custom_script", "custom_script_args"],
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
            let mut custom_script_params = Vec::new();
            let mut custom_script_type_id = None;
            match action {
                b"create_sub_account" => {
                    let custom_script = data_parser::sub_account_cell::get_custom_script(&input_sub_account_data);
                    let account_cell_index;
                    let account_cell_source;
                    let account_cell_witness;
                    let account_cell_reader;

                    match custom_script {
                        Some(val) if val.len() > 0 && val != &[0u8; 33] => {
                            debug!("Found custom scripts in SubAccountCell.data, try to find the AccountCell from cell_deps.");

                            let dep_account_cells = util::find_cells_by_type_id(
                                ScriptType::Type,
                                config_main.type_id_table().account_cell(),
                                Source::CellDep,
                            )?;

                            verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

                            account_cell_index = dep_account_cells[0];
                            account_cell_source = Source::CellDep;

                            account_cell_witness =
                                util::parse_account_cell_witness(&parser, dep_account_cells[0], Source::CellDep)?;
                            account_cell_reader = account_cell_witness.as_reader();

                            verifiers::common::verify_cell_number_and_position(
                                "SubAccountCell",
                                &input_sub_account_cells,
                                &[0],
                                &output_sub_account_cells,
                                &[0],
                            )?;

                            verify_sub_account_cell_is_consistent(
                                input_sub_account_cells[0],
                                output_sub_account_cells[0],
                                vec!["smt_root", "das_profit", "owner_profit"],
                            )?;

                            debug!("Push action into custom_script_params.");

                            let action_str = String::from_utf8(action.to_vec()).unwrap();
                            custom_script_params.push(action_str);

                            debug!(
                                "Try to find the QuoteCell from cell_deps and push quote into custom_script_params."
                            );

                            let quote = util::load_oracle_data(OracleCellType::Quote)?;
                            custom_script_params.push(util::hex_string(&quote.to_le_bytes()));

                            let input_das_profit =
                                data_parser::sub_account_cell::get_das_profit(&input_sub_account_data).unwrap();
                            let output_das_profit =
                                data_parser::sub_account_cell::get_das_profit(&output_sub_account_data).unwrap();
                            let input_owner_profit =
                                data_parser::sub_account_cell::get_owner_profit(&input_sub_account_data).unwrap();
                            let output_owner_profit =
                                data_parser::sub_account_cell::get_owner_profit(&output_sub_account_data).unwrap();
                            let owner_profit = output_owner_profit - input_owner_profit;
                            let das_profit = output_das_profit - input_das_profit;

                            custom_script_params.push(util::hex_string(&owner_profit.to_le_bytes()));
                            custom_script_params.push(util::hex_string(&das_profit.to_le_bytes()));

                            let custom_script_args =
                                data_parser::sub_account_cell::get_custom_script_args(&input_sub_account_data).unwrap();
                            custom_script_params.push(util::hex_string(custom_script_args));

                            let mut type_id = [0u8; 32];
                            if val[0] == ScriptHashType::Type as u8 {
                                // Treat the bytes as args of type ID.
                                let type_of_custom_script = Script::new_builder()
                                    .code_hash(Hash::from(TYPE_ID_CODE_HASH))
                                    .hash_type(Byte::from(val[0]))
                                    .args(Bytes::from(&val[1..]))
                                    .build();
                                type_id = util::blake2b_256(type_of_custom_script.as_slice());
                            } else {
                                // Treat the bytes as code_hash directly.
                                type_id.copy_from_slice(&val[1..]);
                            }

                            debug!("The type ID of custom script is: 0x{}", util::hex_string(&type_id));

                            custom_script_type_id = Some(type_id);
                        }
                        Some(_) | None => {
                            debug!("Do not find custom scripts in SubAccountCell.data, find the AccountCell from inputs and outputs.");

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

                            account_cell_index = input_account_cells[0];
                            account_cell_source = Source::Input;

                            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
                            let input_balance_cells =
                                util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
                            let all_cells = [input_account_cells.clone(), input_balance_cells].concat();
                            verifiers::misc::verify_no_more_cells_with_same_lock(
                                sender_lock.as_reader(),
                                &all_cells,
                                Source::Input,
                            )?;

                            account_cell_witness =
                                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
                            account_cell_reader = account_cell_witness.as_reader();
                            let output_account_cell_witness =
                                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
                            let output_account_cell_reader = output_account_cell_witness.as_reader();

                            verifiers::account_cell::verify_sub_account_enabled(
                                &account_cell_reader,
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
                                &account_cell_reader,
                                &output_account_cell_reader,
                                None,
                                vec![],
                                vec![],
                            )?;

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
                                vec!["smt_root", "das_profit"],
                            )?;
                        }
                    }

                    verifiers::account_cell::verify_status(
                        &account_cell_reader,
                        AccountStatus::Normal,
                        account_cell_index,
                        account_cell_source,
                    )?;

                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        account_cell_index,
                        account_cell_source,
                        timestamp,
                    )?;

                    verifiers::sub_account_cell::verify_sub_account_parent_id(
                        input_sub_account_cells[0],
                        Source::Input,
                        account_cell_reader.id().raw_data(),
                    )?;

                    parent_account = account_cell_reader.account().as_readable();
                    parent_account.extend(ACCOUNT_SUFFIX.as_bytes());
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

                                das_assert!(
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

                                let expired_at = u64::from(sub_account_reader.expired_at());
                                let registered_at = u64::from(sub_account_reader.registered_at());
                                let expiration_years = (expired_at - registered_at) / YEAR_SEC;

                                if custom_script_type_id.is_some() {
                                    debug!("Record registered years in all sub-accounts and pass them to custom scripts later.");
                                    let mut custom_script_param = expiration_years.to_le_bytes().to_vec();
                                    custom_script_param.append(&mut sub_account_reader.account().as_slice().to_vec());
                                    custom_script_params.push(util::hex_string(&custom_script_param));
                                }

                                debug!("Sum basic profit base on registered years in all sub-accounts.");
                                // This variable will be treat as the minimal profit to DAS no matter the custom script exist or not.
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

                                            das_assert!(
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

                                            das_assert!(
                                                current_owner_type == new_owner_type
                                                    && current_owner_args == new_owner_args,
                                                Error::SubAccountEditLockError,
                                                "witnesses[{}] The owner fields in args should be consistent.",
                                                witness.index
                                            );

                                            das_assert!(
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
                    if let Some(type_id) = custom_script_type_id {
                        verify_there_is_only_one_lock_for_normal_cells(
                            input_sub_account_cells[0],
                            output_sub_account_cells[0],
                        )?;

                        verify_profit_to_das_with_custom_script(
                            config_sub_account,
                            profit_to_das,
                            &input_sub_account_data,
                            &output_sub_account_data,
                        )?;

                        debug!("Execute custom script by type ID: 0x{}", util::hex_string(&type_id));
                        let params_with_nul = custom_script_params
                            .iter()
                            .map(|val| {
                                let mut ret = val.as_bytes().to_vec();
                                ret.push(0);
                                ret
                            })
                            .collect::<Vec<_>>();
                        let mut total_size = 0;
                        let params = params_with_nul
                            .iter()
                            .map(|val| unsafe {
                                total_size += val.len();
                                CStr::from_bytes_with_nul_unchecked(val.as_slice())
                            })
                            .collect::<Vec<_>>();
                        debug!("The total size of custom script params: {} bytes", total_size);
                        high_level::exec_cell(&type_id, ScriptHashType::Type, 0, 0, &params).map_err(Error::from)?;
                    } else {
                        verify_profit_to_das(
                            action,
                            output_sub_account_cells[0],
                            &input_sub_account_data,
                            &output_sub_account_data,
                            profit_to_das,
                        )?;
                    }
                }
                _ => {}
            }
        }
        b"collect_sub_account_profit" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_sub_account = parser.configs.sub_account()?;

            debug!("Try to find the SubAccountCells from cell_deps ...");

            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verifiers::common::verify_cell_number_and_position(
                "SubAccountCell",
                &input_sub_account_cells,
                &[0],
                &output_sub_account_cells,
                &[0],
            )?;

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

            verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["das_profit", "owner_profit"],
            )?;

            debug!("Try to find the AccountCell from cell_deps ...");

            let dep_account_cells = util::find_cells_by_type_id(
                ScriptType::Type,
                config_main.type_id_table().account_cell(),
                Source::CellDep,
            )?;

            verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

            let account_cell_witness =
                util::parse_account_cell_witness(&parser, dep_account_cells[0], Source::CellDep)?;
            let account_cell_reader = account_cell_witness.as_reader();

            verifiers::sub_account_cell::verify_sub_account_parent_id(
                input_sub_account_cells[0],
                Source::Input,
                account_cell_reader.id().raw_data(),
            )?;

            let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_sub_account_data).unwrap();
            let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_sub_account_data).unwrap();
            let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_sub_account_data).unwrap();
            let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_sub_account_data).unwrap();

            das_assert!(
                input_das_profit != 0 || input_owner_profit != 0,
                Error::InvalidTransactionStructure,
                "Either the profit of DAS or the profit of owner should not be 0 ."
            );

            let mut collected = false;
            let mut expected_remain_capacity = input_sub_account_capacity;
            let transaction_fee = u64::from(config_sub_account.common_fee());

            if input_das_profit > 0 && output_das_profit == 0 {
                debug!("The profit of DAS has been collected.");

                collected = true;
                expected_remain_capacity -= input_das_profit;

                verifiers::common::verify_das_get_change(input_das_profit)?;
            } else {
                debug!("The profit of DAS is not collected completely, so skip counting it.")
            }

            if input_owner_profit > 0 && output_owner_profit == 0 {
                debug!("The profit of owner has been collected.");

                collected = true;
                expected_remain_capacity -= input_owner_profit;

                let owner_lock = util::derive_owner_lock_from_cell(dep_account_cells[0], Source::CellDep)?;
                verifiers::misc::verify_user_get_change(config_main, owner_lock.as_reader(), input_owner_profit)?;
            } else {
                debug!("The profit of owner is not collected completely, so skip counting it.")
            }

            debug!("Verify if the collection is completed properly.");

            das_assert!(
                collected,
                Error::InvalidTransactionStructure,
                "All profit should be collected at one time, either from DAS or the owner."
            );

            // manual::verify_remain_capacity
            das_assert!(
                output_sub_account_capacity >= expected_remain_capacity - transaction_fee,
                Error::SubAccountCollectProfitError,
                "The capacity of SubAccountCell in outputs should be at least {}, but only {} found.",
                expected_remain_capacity - transaction_fee,
                output_sub_account_capacity
            );
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

    das_assert!(
        input_capacity >= input_das_profit + input_owner_profit + basic_capacity,
        Error::SubAccountCellCapacityError,
        "inputs[{}] The capacity of SubAccountCell should contains profit and basic_capacity, but its not enough.(expected_capacity: {}, current_capacity: {}, das_profit: {}, owner_profit: {})",
        input_index,
        input_das_profit + input_owner_profit + basic_capacity,
        input_capacity,
        input_das_profit,
        input_owner_profit
    );
    das_assert!(
        output_capacity >= output_das_profit + output_owner_profit + basic_capacity,
        Error::SubAccountCellCapacityError,
        "outputs[{}] The capacity of SubAccountCell should contains profit and basic_capacity, but its not enough.(expected_capacity: {}, current_capacity: {}, das_profit: {}, owner_profit: {})",
        output_index,
        output_das_profit + output_owner_profit + basic_capacity,
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

    das_assert!(
        input_remain_fees <= fee + output_remain_fees,
        Error::TxFeeSpentError,
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

    das_assert!(
        util::is_entity_eq(&input_sub_account_cell_lock, &output_sub_account_cell_lock),
        Error::SubAccountCellConsistencyError,
        "The SubAccountCell.lock should be consistent in inputs and outputs."
    );

    let input_sub_account_cell_type =
        high_level::load_cell_type(input_sub_account_cell, Source::Input)?.expect("The type script should exist.");
    let output_sub_account_cell_type =
        high_level::load_cell_type(output_sub_account_cell, Source::Output)?.expect("The type script should exist.");

    das_assert!(
        util::is_entity_eq(&input_sub_account_cell_type, &output_sub_account_cell_type),
        Error::SubAccountCellConsistencyError,
        "The SubAccountCell.type should be consistent in inputs and outputs."
    );

    let input_sub_account_data = high_level::load_cell_data(input_sub_account_cell, Source::Input)?;
    let output_sub_account_data = high_level::load_cell_data(output_sub_account_cell, Source::Output)?;

    macro_rules! assert_field_consistent_if_not_except {
        ($field_name:expr, $get_name:ident) => {
            if !except.contains(&$field_name) {
                let input_value = data_parser::sub_account_cell::$get_name(&input_sub_account_data);
                let output_value = data_parser::sub_account_cell::$get_name(&output_sub_account_data);

                das_assert!(
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
    assert_field_consistent_if_not_except!("custom_script_args", get_custom_script_args);

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

    das_assert!(
        first_root == Some(first_root_in_witnesses),
        Error::SubAccountWitnessSMTRootError,
        "The first SMT root in sub-account witnesses should be equal to the SubAccountCell.data in inputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(first_root.or(Some(&[])).unwrap()),
        util::hex_string(first_root_in_witnesses)
    );

    let data = high_level::load_cell_data(output_sub_account_cell, Source::Output)?;
    let last_root = data_parser::sub_account_cell::get_smt_root(&data);

    das_assert!(
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
    input_data: &[u8],
    output_data: &[u8],
    profit_to_das: u64,
) -> Result<(), Error> {
    debug!("Verify the profit to DAS is recorded properly.");

    if action == b"create_sub_account" {
        let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
        let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();

        das_assert!(
            output_das_profit == input_das_profit + profit_to_das,
            Error::SubAccountProfitError,
            "outputs[{}] The profit of SubAccountCell should contains the new register fees. (input_das_profit: {}, output_das_profit: {}, expected_register_fee: {})",
            cell_index,
            input_das_profit,
            output_das_profit,
            profit_to_das
        );
    } else {
        // TODO Implement withdraw action
        todo!();
    }

    Ok(())
}

fn verify_profit_to_das_with_custom_script(
    config_sub_account: ConfigCellSubAccountReader,
    minimal_profit_to_das: u64,
    input_data: &[u8],
    output_data: &[u8],
) -> Result<(), Error> {
    debug!("Verify the profit to DAS is calculated from rate of config properly.");

    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();
    let owner_profit = output_owner_profit - input_owner_profit;
    let das_profit = output_das_profit - input_das_profit;
    let total_profit = owner_profit + das_profit;
    let profit_rate = u32::from(config_sub_account.new_sub_account_custom_price_das_profit_rate());

    das_assert!(
        das_profit >= minimal_profit_to_das,
        Error::SubAccountProfitError,
        "The profit to DAS should be greater than or equal to the minimal profit which is 1 CKB per account. (das_profit: {}, minimal_profit_to_das: {})",
        das_profit,
        minimal_profit_to_das
    );

    // CAREFUL: Overflow risk
    let mut expected_das_profit = total_profit * profit_rate as u64 / RATE_BASE;
    if expected_das_profit < minimal_profit_to_das {
        expected_das_profit = minimal_profit_to_das;
    }

    das_assert!(
        expected_das_profit == das_profit,
        Error::SubAccountProfitError,
        "The profit to DAS should be calculated from rate of config properly. (expected_das_profit: {}, das_profit: {})",
        expected_das_profit,
        das_profit
    );

    Ok(())
}

fn verify_there_is_only_one_lock_for_normal_cells(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
) -> Result<(), Error> {
    debug!("Verify there is only one lock for cells which is not SubAccountCell.");

    let mut lock_hash = None;
    for (field_name, source) in [("inputs", Source::Input), ("outputs", Source::Output)] {
        let mut i = 0;
        loop {
            if source == Source::Input && i == input_sub_account_cell {
                i += 1;
                continue;
            } else if source == Source::Output && i == output_sub_account_cell {
                i += 1;
                continue;
            }

            let ret = high_level::load_cell_lock_hash(i, source);
            match ret {
                Ok(val) => {
                    if lock_hash.is_none() {
                        lock_hash = Some(val);
                    } else {
                        das_assert!(
                            lock_hash == Some(val),
                            Error::SubAccountNormalCellLockLimit,
                            "{}[{}] There should be only one lock for cells which is not SubAccountCell.",
                            field_name,
                            i
                        );
                    }
                }
                Err(SysError::IndexOutOfBound) => {
                    break;
                }
                Err(err) => {
                    return Err(Error::from(err));
                }
            }

            i += 1;
        }
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
