use alloc::vec::Vec;
use alloc::{boxed::Box, format, string::String, vec};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, high_level};
use das_core::{
    assert,
    constants::{das_wallet_lock, DasLockType, OracleCellType, ScriptType, TypeScript},
    data_parser, debug,
    eip712::{to_semantic_address, verify_eip712_hashes},
    error::Error,
    util, verifiers, warn,
    witness_parser::WitnessesParser,
};
use das_map::{map::Map, util as map_util};
use das_types::{
    constants::{AccountStatus, DataType, LockRole, SubAccountEnableStatus},
    mixer::*,
    packed::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    if action != b"init_account_chain" {
        util::is_system_off(&parser)?;
    }

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    match action {
        b"init_account_chain" => {
            unreachable!();
        }
        b"transfer_account" | b"edit_manager" | b"edit_records" => {
            verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            assert!(
                input_account_cells.len() == 1 && output_account_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be only one AccountCell in inputs and outputs."
            );

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            verifiers::misc::verify_no_more_cells_with_same_lock(
                sender_lock.as_reader(),
                &input_account_cells,
                Source::Input,
            )?;

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            match action {
                b"transfer_account" => {
                    verify_eip712_hashes(&parser, transfer_account_to_semantic)?;

                    let config_account = parser.configs.account()?;

                    verify_transaction_fee_spent_correctly(
                        action,
                        config_account,
                        input_account_cells[0],
                        output_account_cells[0],
                    )?;
                    verify_action_throttle(
                        action,
                        config_account,
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_cell_status(
                        &input_cell_witness_reader,
                        AccountStatus::Normal,
                        input_account_cells[0],
                        Source::Input,
                    )?;
                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        Some("owner"),
                        vec![],
                        vec!["last_transfer_account_at", "records"],
                    )?;
                    verifiers::account_cell::verify_account_witness_record_empty(
                        &output_cell_witness_reader,
                        output_account_cells[0],
                        Source::Output,
                    )?;
                }
                b"edit_manager" => {
                    verify_eip712_hashes(&parser, edit_manager_to_semantic)?;

                    let config_account = parser.configs.account()?;

                    verify_transaction_fee_spent_correctly(
                        action,
                        config_account,
                        input_account_cells[0],
                        output_account_cells[0],
                    )?;
                    verify_action_throttle(
                        action,
                        config_account,
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_cell_status(
                        &input_cell_witness_reader,
                        AccountStatus::Normal,
                        input_account_cells[0],
                        Source::Input,
                    )?;
                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        Some("manager"),
                        vec![],
                        vec!["last_edit_manager_at"],
                    )?;
                }
                b"edit_records" => {
                    verify_eip712_hashes(&parser, edit_records_to_semantic)?;

                    let config_account = parser.configs.account()?;

                    verify_transaction_fee_spent_correctly(
                        action,
                        config_account,
                        input_account_cells[0],
                        output_account_cells[0],
                    )?;
                    verify_action_throttle(
                        action,
                        config_account,
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_cell_status(
                        &input_cell_witness_reader,
                        AccountStatus::Normal,
                        input_account_cells[0],
                        Source::Input,
                    )?;
                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        None,
                        vec![],
                        vec!["records", "last_edit_records_at"],
                    )?;
                    verifiers::account_cell::verify_records_keys(&parser, output_cell_witness_reader.records())?;
                }
                _ => unreachable!(),
            }
        }
        b"renew_account" => {
            parser.parse_cell()?;

            let prices = parser.configs.price()?.prices();
            let config_main = parser.configs.main()?;
            let config_income = parser.configs.income()?;
            let income_cell_type_id = config_main.type_id_table().income_cell();

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            assert!(
                input_account_cells.len() == 1 && output_account_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be only one AccountCell in inputs and outputs."
            );

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec!["expired_at"],
                vec![],
            )?;

            debug!("Check if IncomeCells in this transaction is correct.");

            let input_income_cells = util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Input)?;
            let output_income_cells =
                util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Output)?;

            assert!(
                input_income_cells.len() <= 1,
                Error::InvalidTransactionStructure,
                "The number of IncomeCells in inputs should be less than or equal to 1. (expected: <= 1, current: {})",
                input_income_cells.len()
            );

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
            let all_cells = [
                input_account_cells.clone(),
                input_income_cells.clone(),
                balance_cells.clone(),
            ]
            .concat();
            verifiers::misc::verify_no_more_cells_with_same_lock(sender_lock.as_reader(), &all_cells, Source::Input)?;

            debug!("Verify if the profit is distribute correctly.");

            assert!(
                output_income_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "The number of IncomeCells in outputs should be exactly 1. (expected: == 1, current: {})",
                output_income_cells.len()
            );

            verifiers::misc::verify_always_success_lock(output_income_cells[0], Source::Output)?;

            let (_, _, entity) =
                parser.verify_and_get(DataType::IncomeCellData, output_income_cells[0], Source::Output)?;
            let income_cell_witness = IncomeCellData::from_slice(entity.as_reader().raw_data())
                .map_err(|_| Error::WitnessEntityDecodingError)?;
            let income_cell_witness_reader = income_cell_witness.as_reader();

            let mut profit_map = Map::new();
            let das_wallet_lock = Script::from(das_wallet_lock());

            let paid = if income_cell_witness_reader.records().len() == 1 {
                u64::from(income_cell_witness_reader.records().get(0).unwrap().capacity())
            } else if income_cell_witness_reader.records().len() == 2 {
                u64::from(income_cell_witness_reader.records().get(1).unwrap().capacity())
            } else {
                warn!("The IncomeCell should contain at most two records in this transaction.");
                return Err(Error::InvalidTransactionStructure);
            };
            map_util::add(&mut profit_map, das_wallet_lock.as_slice().to_vec(), paid);

            verifiers::income_cell::verify_records_match_with_creating(
                config_income,
                output_income_cells[0],
                Source::Output,
                income_cell_witness_reader,
                profit_map,
            )?;

            debug!("Check if the renewal duration is longer than or equal to one year.");

            let input_data = util::load_cell_data(input_account_cells[0], Source::Input)?;
            let output_data = util::load_cell_data(output_account_cells[0], Source::Output)?;
            let input_expired_at = data_parser::account_cell::get_expired_at(&input_data);
            let output_expired_at = data_parser::account_cell::get_expired_at(&output_data);
            let duration = output_expired_at - input_expired_at;

            assert!(
                duration >= 365 * 86400,
                Error::AccountCellRenewDurationMustLongerThanYear,
                "The AccountCell renew should be longer than 1 year. (current: {}, expected: >= 31_536_000)",
                duration
            );

            debug!("Check if the expired_at field has been updated correctly based on the capacity paid by the user.");

            let length_in_price = util::get_length_in_price(output_cell_witness_reader.account().len() as u64);
            // Find out register price in from ConfigCellRegister.
            let price = prices
                .iter()
                .find(|item| u8::from(item.length()) == length_in_price)
                .ok_or(Error::ItemMissing)?;

            let renew_price_in_usd = u64::from(price.renew()); // x USD
            let quote = util::load_oracle_data(OracleCellType::Quote)?;

            let yearly_capacity = util::calc_yearly_capacity(renew_price_in_usd, quote, 0);
            assert!(
                paid >= yearly_capacity,
                Error::AccountCellRenewDurationMustLongerThanYear,
                "The paid capacity should be at least 1 year. (current: {}, expected: >= {}",
                paid,
                yearly_capacity
            );

            // Renew price for 1 year in CKB = x ÷ y .
            let expected_duration = util::calc_duration_from_paid(paid, renew_price_in_usd, quote, 0);
            // The duration can be floated within the range of one day.
            assert!(
                duration >= expected_duration - 86400 && duration <= expected_duration + 86400,
                Error::AccountCellRenewDurationBiggerThanPayed,
                "The duration should be equal to {} +/- 86400s. (current: duration({}), calculation: (paid({}) / (renew_price({}) / quote({}) * 100_000_000) ) * 86400 * 365)",
                expected_duration,
                duration,
                paid,
                renew_price_in_usd,
                quote
            );

            debug!("Verify if sender get their change properly.");

            let total_input_capacity = util::load_cells_capacity(&balance_cells, Source::Input)?;

            if total_input_capacity > paid {
                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    total_input_capacity - paid,
                )?;
            }

            // The AccountCell can be used as long as it is not modified.
        }
        b"confirm_proposal" => {
            util::require_type_script(
                &parser,
                TypeScript::ProposalCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"recycle_expired_account_by_keeper" => {
            return Err(Error::InvalidTransactionStructure);
        }
        b"start_account_sale" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountSaleCellType,
                Source::Output,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"cancel_account_sale" | b"buy_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountSaleCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"accept_offer" => {
            util::require_type_script(
                &parser,
                TypeScript::OfferCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"force_recover_account_status" => {
            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            assert!(
                input_cells.len() == 1 && output_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be one AccountCell in outputs and one in inputs."
            );
            assert!(
                input_cells[0] == 0 && output_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "The AccountCells should only appear at inputs[0] and outputs[0]."
            );

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(&parser, output_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            debug!("Verify if the AccountCell is consistent in inputs and outputs.");

            verifiers::account_cell::verify_account_capacity_not_decrease(input_cells[0], output_cells[0])?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_cells[0],
                output_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec![],
                vec!["status"],
            )?;

            debug!("Verify if the AccountCell status updated correctly.");

            let input_status = u8::from(input_cell_witness_reader.status());
            assert!(
                input_status != AccountStatus::Normal as u8,
                Error::InvalidTransactionStructure,
                "The AccountCell in inputs should not be in NORMAL status."
            );

            let output_status = u8::from(output_cell_witness_reader.status());
            assert!(
                output_status == AccountStatus::Normal as u8,
                Error::InvalidTransactionStructure,
                "The AccountCell in outputs should be in NORMAL status."
            );

            debug!("Verify if the AccountCell is actually expired.");

            let input_cell_data = high_level::load_cell_data(input_cells[0], Source::Input)?;
            let expired_at = data_parser::account_cell::get_expired_at(&input_cell_data);
            let account = data_parser::account_cell::get_account(&input_cell_data);

            // It is a convention that the deal can be canceled immediately when expiring.
            assert!(
                timestamp > expired_at,
                Error::AccountCellIsNotExpired,
                "The AccountCell is still not expired."
            );

            let capacity_should_recycle;
            let cell;
            if input_status == AccountStatus::Selling as u8 {
                let type_id = parser.configs.main()?.type_id_table().account_sale_cell();
                let (input_sale_cells, output_sale_cells) =
                    util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id)?;

                verifiers::common::verify_removed_cell_in_correct_position(
                    "AccountSaleCell",
                    &input_sale_cells,
                    &output_sale_cells,
                    Some(1),
                )?;

                let cell_witness = util::parse_account_sale_cell_witness(&parser, input_sale_cells[0], Source::Input)?;
                let cell_witness_reader = cell_witness.as_reader();

                assert!(
                    account == cell_witness_reader.account().raw_data(),
                    Error::AccountSaleCellAccountIdInvalid,
                    "The account in AccountCell and AccountSaleCell should be the same."
                );

                cell = input_sale_cells[0];
            } else {
                // TODO Verify the account in AccountCell and AccountAuctionCell is the same.
                cell = 0;
            }
            capacity_should_recycle = high_level::load_cell_capacity(cell, Source::Input)?;

            debug!(
                "Found the capacity should be recycled is {} shannon.",
                capacity_should_recycle
            );

            let balance_cell_type_id = config_main.type_id_table().balance_cell();
            let (input_balance_cells, outputs_balance_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, balance_cell_type_id)?;

            verifiers::common::verify_created_cell_in_correct_position(
                "BalanceCell",
                &input_balance_cells,
                &outputs_balance_cells,
                Some(1),
            )?;

            let expected_lock = util::derive_owner_lock_from_cell(input_cells[0], Source::Input)?;
            let current_lock = high_level::load_cell_lock(outputs_balance_cells[0], Source::Output)?.into();
            assert!(
                util::is_entity_eq(&expected_lock, &current_lock),
                Error::AccountSaleCellRefundError,
                "The lock receiving the refund is incorrect.(expected: {}, current: {})",
                expected_lock,
                current_lock
            );

            let expected_capacity = capacity_should_recycle - 10_000;
            let current_capacity = high_level::load_cell_capacity(outputs_balance_cells[0], Source::Output)?;
            assert!(
                current_capacity >= expected_capacity,
                Error::AccountSaleCellRefundError,
                "The capacity refunding is incorrect.(expected: {}, current: {})",
                expected_capacity,
                current_capacity
            );
        }
        b"enable_sub_account" => {
            // CAREFUL! This action is intentionally ignoring EIP712 verification.
            // verify_eip712_hashes(&parser, transfer_account_to_semantic)?;

            verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            assert!(
                input_account_cells.len() == 1 && input_account_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "There should be one AccountCell at inputs[0]."
            );
            assert!(
                output_account_cells.len() == 1 && output_account_cells[0] == 0,
                Error::InvalidTransactionStructure,
                "There should be one AccountCell at outputs[0]."
            );

            let input_account_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_witness_reader = input_account_witness.as_reader();
            let output_account_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_witness_reader = output_account_witness.as_reader();

            debug!("Verify if the AccountCell is in beta list.");

            let beta_list: Vec<&[u8]> = vec![b"xxxxx.bit", b"xxxx.bit"];
            let account = util::get_account_from_reader(&input_account_witness_reader);

            assert!(
                beta_list.contains(&account.as_bytes()),
                Error::SubAccountJoinBetaError,
                "The account is not allow to enable sub-account feature in beta test."
            );

            debug!("Verify if the AccountCell is locked or expired.");

            verifiers::account_cell::verify_account_cell_status(
                &input_account_witness_reader,
                AccountStatus::Normal,
                input_account_cells[0],
                Source::Input,
            )?;
            verifiers::account_cell::verify_account_expiration(config_account, input_account_cells[0], timestamp)?;

            debug!("Verify if every aspects of the AccountCell is consistent.");

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_account_witness_reader,
                &output_account_witness_reader,
                None,
                vec![],
                vec!["enable_sub_account"],
            )?;

            debug!("Verify if the AccountCell can enable sub-account function.");

            match input_account_witness_reader.try_into_latest() {
                Ok(reader) => {
                    let enable_status = u8::from(reader.enable_sub_account());
                    assert!(
                        enable_status == SubAccountEnableStatus::On as u8,
                        Error::AccountCellPermissionDenied,
                        "{:?}[{}]Only AccountCells with enable_sub_account field is {} can enable its sub-account function.",
                        Source::Output,
                        output_account_cells[0],
                        SubAccountEnableStatus::Off as u8
                    );
                }
                Err(_) => {
                    // If the AccountCell in inputs is old version, it definitely have not enabled the sub-account function.
                }
            }

            match output_account_witness_reader.try_into_latest() {
                Ok(reader) => {
                    let enable_status = u8::from(reader.enable_sub_account());
                    assert!(
                        enable_status == SubAccountEnableStatus::On as u8,
                        Error::AccountCellPermissionDenied,
                        "{:?}[{}]The AccountCell.enable_sub_account should be {} .",
                        Source::Output,
                        output_account_cells[0],
                        SubAccountEnableStatus::On as u8
                    );
                }
                Err(_) => {
                    warn!(
                        "{:?}[{}]The version of this AccountCell should be latest.",
                        Source::Output,
                        output_account_cells[0]
                    );
                    return Err(Error::InvalidTransactionStructure);
                }
            }

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
            let all_cells = [input_account_cells.clone(), balance_cells.clone()].concat();
            verifiers::misc::verify_no_more_cells_with_same_lock(sender_lock.as_reader(), &all_cells, Source::Input)?;

            debug!("Verify if the SubAccountCell is created properly.");

            let sub_account_cell_type_id = config_main.type_id_table().sub_account_cell();
            let (input_sub_account_cells, output_sub_account_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, sub_account_cell_type_id)?;

            verifiers::common::verify_created_cell_in_correct_position(
                "SubAccountCell",
                &input_sub_account_cells,
                &output_sub_account_cells,
                Some(1),
            )?;

            verifiers::misc::verify_always_success_lock(output_sub_account_cells[0], Source::Output)?;

            let sub_account_cell_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let expected_capacity =
                u64::from(config_sub_account.basic_capacity()) + u64::from(config_sub_account.prepared_fee_capacity());

            assert!(
                sub_account_cell_capacity == expected_capacity,
                Error::SubAccountCellCapacityError,
                "The initial capacity of SubAccountCell should be equal to ConfigCellSubAccount.basic_capacity + ConfigCellSubAccount.prepared_fee_capacity .(expected: {}, current: {})",
                expected_capacity,
                sub_account_cell_capacity
            );

            let type_script = high_level::load_cell_type(output_sub_account_cells[0], Source::Output)?.unwrap();
            let account_id = type_script.as_reader().args().raw_data();
            let expected_account_id = output_account_witness_reader.id().raw_data();

            assert!(
                account_id == expected_account_id,
                Error::SubAccountCellAccountIdError,
                "The type.args of SubAccountCell should be the same with the AccountCell.witness.id .(expected: {}, current: {})",
                util::hex_string(expected_account_id),
                util::hex_string(account_id)
            );

            let sub_account_outputs_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;
            let empty_smt_root = vec![
                0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];

            assert!(
                empty_smt_root == sub_account_outputs_data,
                Error::SubAccountCellSMTRootError,
                "The SMT root of SubAccountCell should be empty."
            );

            debug!("Verify if sender get their change properly.");

            let total_input_capacity = util::load_cells_capacity(&balance_cells, Source::Input)?;

            if total_input_capacity > sub_account_cell_capacity {
                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    total_input_capacity - sub_account_cell_capacity,
                )?;
            }
        }
        b"create_sub_account" => {
            util::require_type_script(
                &parser,
                TypeScript::SubAccountCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn transfer_account_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // Parse from address from the AccountCell's lock script in inputs.
    // let from_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
    // let from_address = to_semantic_address(from_lock.as_reader().into(), 1..21)?;
    // Parse to address from the AccountCell's lock script in outputs.
    let to_lock = high_level::load_cell_lock(output_cells[0], Source::Output)?;
    let to_address = to_semantic_address(parser, to_lock.as_reader().into(), LockRole::Owner)?;

    Ok(format!("TRANSFER THE ACCOUNT {} TO {}", account, to_address))
}

fn edit_manager_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("EDIT MANAGER OF ACCOUNT {}", account))
}

fn edit_records_to_semantic(parser: &WitnessesParser) -> Result<String, Error> {
    let type_id_table_reader = parser.configs.main()?.type_id_table();
    let (input_cells, _output_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id_table_reader.account_cell())?;

    // Parse account from the data of the AccountCell in inputs.
    let data_in_bytes = util::load_cell_data(input_cells[0], Source::Input)?;
    let account_in_bytes = data_parser::account_cell::get_account(&data_in_bytes);
    let account = String::from_utf8(account_in_bytes.to_vec()).map_err(|_| Error::EIP712SerializationError)?;

    // TODO Improve semantic message of this transaction.
    Ok(format!("EDIT RECORDS OF ACCOUNT {}", account))
}

fn verify_transaction_fee_spent_correctly(
    action: &[u8],
    config: ConfigCellAccountReader,
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if the fee in the AccountCell is spent correctly.");

    // TODO MIXIN Fix this with new data structure.
    let lock = high_level::load_cell_lock(input_account_index, Source::Input)?;
    let lock_type = data_parser::das_lock_args::get_owner_type(lock.as_reader().args().raw_data());
    let basic_capacity = if lock_type == DasLockType::MIXIN as u8 {
        23_000_000_000u64
    } else {
        u64::from(config.basic_capacity())
    };

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let account_length = data_parser::account_cell::get_account(&input_data).len() as u64;

    let fee = match action {
        b"transfer_account" => u64::from(config.transfer_account_fee()),
        b"edit_manager" => u64::from(config.edit_manager_fee()),
        b"edit_records" => u64::from(config.edit_records_fee()),
        _ => return Err(Error::ActionNotSupported),
    };
    let storage_capacity = basic_capacity + account_length * 100_000_000;

    verifiers::common::verify_tx_fee_spent_correctly(
        "AccountCell",
        input_account_index,
        output_account_index,
        fee,
        storage_capacity,
    )?;

    Ok(())
}

fn verify_action_throttle<'a>(
    action: &[u8],
    config: ConfigCellAccountReader,
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    current_timestamp: u64,
) -> Result<(), Error> {
    macro_rules! assert_action_throttle {
        ($input_witness_reader:expr, $output_witness_reader:expr, $config_field:ident, $field:ident, $field_name:expr) => {{
            let throttle = u32::from(config.$config_field()) as u64;
            let prev = u64::from($input_witness_reader.$field());
            let current = u64::from($output_witness_reader.$field());

            if prev != 0 {
                assert!(
                    current >= prev + throttle,
                    Error::AccountCellThrottle,
                    "The AccountCell is used too often, need to wait {} seconds between each transaction.(current: {}, prev: {})",
                    throttle,
                    current,
                    prev
                );
            }

            assert!(
                current_timestamp == current,
                Error::AccountCellThrottle,
                "The AccountCell.{} in outputs should be the same as the timestamp in the TimeCell.(expected: {}, current: {})",
                $field_name,
                current_timestamp,
                current
            );
        }};
    }

    if input_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(Error::InvalidTransactionStructure);
    } else {
        match action {
            b"transfer_account" => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                transfer_account_throttle,
                last_transfer_account_at,
                "last_transfer_account_at"
            ),
            b"edit_manager" => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                edit_manager_throttle,
                last_edit_manager_at,
                "last_edit_manager_at"
            ),
            b"edit_records" => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                edit_records_throttle,
                last_edit_records_at,
                "last_edit_records_at"
            ),
            _ => return Err(Error::ActionNotSupported),
        }
    }

    Ok(())
}
