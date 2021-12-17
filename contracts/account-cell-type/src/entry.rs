use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, high_level};
use das_core::{
    assert,
    constants::{das_wallet_lock, OracleCellType, ScriptType, TypeScript, CUSTOM_KEYS_NAMESPACE},
    data_parser, debug,
    eip712::{to_semantic_address, verify_eip712_hashes},
    error::Error,
    parse_account_cell_witness, parse_witness, util, verifiers,
    witness_parser::WitnessesParser,
};
use das_types::{
    constants::{AccountStatus, DataType, LockRole},
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
        util::is_system_off(&mut parser)?;
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

            parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellAccount])?;
            parser.parse_cell()?;

            let (input_account_cells, output_account_cells) = load_account_cells()?;
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

            let input_cell_witness: Box<dyn AccountCellDataMixer>;
            let input_cell_witness_reader;
            parse_account_cell_witness!(
                input_cell_witness,
                input_cell_witness_reader,
                parser,
                input_account_cells[0],
                Source::Input
            );

            let output_cell_witness: Box<dyn AccountCellDataMixer>;
            let output_cell_witness_reader;
            parse_account_cell_witness!(
                output_cell_witness,
                output_cell_witness_reader,
                parser,
                output_account_cells[0],
                Source::Output
            );

            assert!(
                output_cell_witness_reader.version() == 2,
                Error::DataTypeUpgradeRequired,
                "The witness of the AccountCell in outputs should be upgrade to version 2."
            );

            match action {
                b"transfer_account" => {
                    verify_eip712_hashes(&parser, transfer_account_to_semantic)?;

                    let config_account = parser.configs.account()?;

                    verify_input_account_must_normal_status(&input_cell_witness_reader)?;
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
                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_lock_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        Some("owner"),
                    )?;
                    verifiers::account_cell::verify_account_data_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        vec![],
                    )?;
                    verifiers::account_cell::verify_account_witness_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
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

                    verify_input_account_must_normal_status(&input_cell_witness_reader)?;
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
                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_lock_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        Some("manager"),
                    )?;
                    verifiers::account_cell::verify_account_data_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        vec![],
                    )?;
                    verifiers::account_cell::verify_account_witness_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        vec!["last_edit_manager_at"],
                    )?;
                }
                b"edit_records" => {
                    verify_eip712_hashes(&parser, edit_records_to_semantic)?;

                    parser.parse_config(&[DataType::ConfigCellRecordKeyNamespace])?;
                    let config_account = parser.configs.account()?;
                    let record_key_namespace = parser.configs.record_key_namespace()?;

                    verify_input_account_must_normal_status(&input_cell_witness_reader)?;
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
                    verifiers::account_cell::verify_account_expiration(
                        config_account,
                        input_account_cells[0],
                        timestamp,
                    )?;
                    verifiers::account_cell::verify_account_lock_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        None,
                    )?;
                    verifiers::account_cell::verify_account_data_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        vec![],
                    )?;
                    verifiers::account_cell::verify_account_witness_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        vec!["records", "last_edit_records_at"],
                    )?;
                    verify_records_keys(config_account, record_key_namespace, &output_cell_witness_reader)?;
                }
                _ => unreachable!(),
            }
        }
        b"renew_account" => {
            parser.parse_cell()?;
            parser.parse_config(&[DataType::ConfigCellAccount, DataType::ConfigCellPrice])?;

            let prices = parser.configs.price()?.prices();
            let config_main = parser.configs.main()?;
            let income_cell_type_id = config_main.type_id_table().income_cell();

            let (input_account_cells, output_account_cells) = load_account_cells()?;
            assert!(
                input_account_cells.len() == 1 && output_account_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be only one AccountCell in inputs and outputs."
            );

            let input_cell_witness: Box<dyn AccountCellDataMixer>;
            let input_cell_witness_reader;
            parse_account_cell_witness!(
                input_cell_witness,
                input_cell_witness_reader,
                parser,
                input_account_cells[0],
                Source::Input
            );

            let output_cell_witness: Box<dyn AccountCellDataMixer>;
            let output_cell_witness_reader;
            parse_account_cell_witness!(
                output_cell_witness,
                output_cell_witness_reader,
                parser,
                output_account_cells[0],
                Source::Output
            );

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;
            verifiers::account_cell::verify_account_lock_consistent(
                input_account_cells[0],
                output_account_cells[0],
                None,
            )?;
            verifiers::account_cell::verify_account_data_consistent(
                input_account_cells[0],
                output_account_cells[0],
                vec!["expired_at"],
            )?;
            verifiers::account_cell::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
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

            let mut expected_first_record = None;
            if input_income_cells.len() == 1 {
                let income_cell_witness;
                let income_cell_witness_reader;
                parse_witness!(
                    income_cell_witness,
                    income_cell_witness_reader,
                    parser,
                    input_income_cells[0],
                    Source::Input,
                    DataType::IncomeCellData,
                    IncomeCellData
                );

                // The IncomeCell should be a newly created cell with only one record which is belong to the creator, but we do not need to check everything here, so we only check the length.
                verifiers::income_cell::verify_newly_created(
                    income_cell_witness_reader,
                    input_income_cells[0],
                    Source::Input,
                )?;

                expected_first_record = income_cell_witness.records().get(0);
            }

            assert!(
                output_income_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "The number of IncomeCells in outputs should be exactly 1. (expected: == 1, current: {})",
                output_income_cells.len()
            );

            verifiers::misc::verify_always_success_lock(output_income_cells[0], Source::Output)?;

            let income_cell_capacity =
                high_level::load_cell_capacity(output_income_cells[0], Source::Output).map_err(|e| Error::from(e))?;
            let (_, _, entity) =
                parser.verify_and_get(DataType::IncomeCellData, output_income_cells[0], Source::Output)?;
            let income_cell_witness = IncomeCellData::from_slice(entity.as_reader().raw_data())
                .map_err(|_| Error::WitnessEntityDecodingError)?;
            let income_cell_witness_reader = income_cell_witness.as_reader();

            let paid;
            let das_wallet_lock = Script::from(das_wallet_lock());
            if let Some(expected_first_record) = expected_first_record {
                // IncomeCell is created before this transaction, so it is include the creator's income record.
                assert!(
                    income_cell_witness_reader.records().len() == 2,
                    Error::InvalidTransactionStructure,
                    "The number of records of IncomeCells in outputs should be exactly 2. (expected: == 2, current: {})",
                    income_cell_witness_reader.records().len()
                );

                let first_record = income_cell_witness_reader.records().get(0).unwrap();
                let exist_capacity = u64::from(first_record.capacity());

                assert!(
                    util::is_reader_eq(expected_first_record.as_reader(), first_record),
                    Error::InvalidTransactionStructure,
                    "The first record of IncomeCell should keep the same as in inputs."
                );

                let second_record = income_cell_witness_reader.records().get(1).unwrap();
                paid = u64::from(second_record.capacity());

                assert!(
                    util::is_reader_eq(second_record.belong_to(), das_wallet_lock.as_reader()),
                    Error::InvalidTransactionStructure,
                    "The second record in IncomeCell should belong to DAS[{}].",
                    das_wallet_lock.as_reader()
                );

                assert!(
                    income_cell_capacity == exist_capacity + paid,
                    Error::InvalidTransactionStructure,
                    "The capacity of IncomeCell in outputs is incorrect. (expected: {}, current: {})",
                    exist_capacity + paid,
                    income_cell_capacity
                );
            } else {
                // IncomeCell is created with only profit.
                assert!(
                    income_cell_witness_reader.records().len() == 1,
                    Error::InvalidTransactionStructure,
                    "The number of records of IncomeCells in outputs should be exactly 1. (expected: == 1, current: {})",
                    income_cell_witness_reader.records().len()
                );

                let first_record = income_cell_witness_reader.records().get(0).unwrap();
                paid = u64::from(first_record.capacity());

                assert!(
                    util::is_reader_eq(first_record.belong_to(), das_wallet_lock.as_reader()),
                    Error::InvalidTransactionStructure,
                    "The only record in IncomeCell should belong to DAS[{}].",
                    das_wallet_lock.as_reader()
                );

                assert!(
                    income_cell_capacity == paid,
                    Error::InvalidTransactionStructure,
                    "The capacity of IncomeCell in outputs is incorrect. (expected: {}, current: {})",
                    paid,
                    income_cell_capacity
                );
            }

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

            let output_account_witness_reader = output_cell_witness_reader
                .try_into_latest()
                .map_err(|_| Error::NarrowMixerTypeFailed)?;
            let length_in_price = util::get_length_in_price(output_account_witness_reader.account().len() as u64);

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

            let mut total_input_capacity = 0;
            for i in balance_cells.iter() {
                total_input_capacity += high_level::load_cell_capacity(*i, Source::Input)?;
            }

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
                &mut parser,
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
                &mut parser,
                TypeScript::AccountSaleCellType,
                Source::Output,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"cancel_account_sale" | b"buy_account" => {
            util::require_type_script(
                &mut parser,
                TypeScript::AccountSaleCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"accept_offer" => {
            util::require_type_script(
                &mut parser,
                TypeScript::OfferCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"force_recover_account_status" => {
            parser.parse_config(&[DataType::ConfigCellMain])?;
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

            let input_cell_witness: Box<dyn AccountCellDataMixer>;
            let input_cell_witness_reader;
            parse_account_cell_witness!(
                input_cell_witness,
                input_cell_witness_reader,
                parser,
                input_cells[0],
                Source::Input
            );

            let output_cell_witness: Box<dyn AccountCellDataMixer>;
            let output_cell_witness_reader;
            parse_account_cell_witness!(
                output_cell_witness,
                output_cell_witness_reader,
                parser,
                output_cells[0],
                Source::Output
            );

            debug!("Verify if the AccountCell is consistent in inputs and outputs.");

            verifiers::account_cell::verify_account_lock_consistent(input_cells[0], output_cells[0], None)?;
            verifiers::account_cell::verify_account_data_consistent(input_cells[0], output_cells[0], vec![])?;
            verifiers::account_cell::verify_account_capacity_not_decrease(input_cells[0], output_cells[0])?;
            verifiers::account_cell::verify_account_witness_consistent(
                input_cells[0],
                output_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                vec!["status"],
            )?;

            if input_cell_witness_reader.version() == 1 {
                // There is no version 1 AccountCell in mainnet, so we simply disable them here.
                return Err(Error::InvalidTransactionStructure);
            } else {
                debug!("Verify if the AccountCell status updated correctly.");

                let input_cell_witness_reader = input_cell_witness_reader
                    .try_into_latest()
                    .map_err(|_| Error::NarrowMixerTypeFailed)?;
                let input_status = u8::from(input_cell_witness_reader.status());
                assert!(
                    input_status != AccountStatus::Normal as u8,
                    Error::InvalidTransactionStructure,
                    "The AccountCell in inputs should not be in NORMAL status."
                );

                let output_cell_witness_reader = output_cell_witness_reader
                    .try_into_latest()
                    .map_err(|_| Error::NarrowMixerTypeFailed)?;
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
                    assert!(
                        input_sale_cells.len() == 1 && output_sale_cells.len() == 0 && input_sale_cells[0] == 1,
                        Error::InvalidTransactionStructure,
                        "There should be only one AccountSaleCell at inputs[1]."
                    );

                    let cell_witness;
                    let cell_witness_reader;
                    parse_witness!(
                        cell_witness,
                        cell_witness_reader,
                        parser,
                        input_sale_cells[0],
                        Source::Input,
                        DataType::AccountSaleCellData,
                        AccountSaleCellData
                    );

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
                assert!(
                    input_balance_cells.len() == 0 && outputs_balance_cells.len() == 1 && outputs_balance_cells[0] == 1,
                    Error::InvalidTransactionStructure,
                    "There should be no BalanceCell in inputs and only one BalanceCell at outputs[1]"
                );

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
    // let from_lock = high_level::load_cell_lock(input_cells[0], Source::Input).map_err(|e| Error::from(e))?;
    // let from_address = to_semantic_address(from_lock.as_reader().into(), 1..21)?;
    // Parse to address from the AccountCell's lock script in outputs.
    let to_lock = high_level::load_cell_lock(output_cells[0], Source::Output).map_err(|e| Error::from(e))?;
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

fn verify_input_account_must_normal_status<'a>(
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let witness_reader = input_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    let account_status = u8::from(witness_reader.status());
    if account_status != (AccountStatus::Normal as u8) {
        return Err(Error::AccountCellStatusLocked);
    }
    return Ok(());
}

fn load_account_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let (input_account_cells, output_account_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

    Ok((input_account_cells, output_account_cells))
}

fn verify_transaction_fee_spent_correctly(
    action: &[u8],
    config: ConfigCellAccountReader,
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if the fee in the AccountCell is spent correctly.");

    let input_capacity =
        high_level::load_cell_capacity(input_account_index, Source::Input).map_err(|e| Error::from(e))?;
    let output_capacity =
        high_level::load_cell_capacity(output_account_index, Source::Output).map_err(|e| Error::from(e))?;

    // The capacity is not changed, skip the following verification.
    if input_capacity == output_capacity {
        return Ok(());
    }

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let account_length = data_parser::account_cell::get_account(&input_data).len() as u64;

    let fee = match action {
        b"transfer_account" => u64::from(config.transfer_account_fee()),
        b"edit_manager" => u64::from(config.edit_manager_fee()),
        b"edit_records" => u64::from(config.edit_records_fee()),
        _ => return Err(Error::ActionNotSupported),
    };
    let storage_capacity = u64::from(config.basic_capacity()) + account_length * 100_000_000;

    assert!(
        output_capacity >= storage_capacity,
        Error::AccountCellNoMoreFee,
        "The AccountCell has no more capacity as fee for this transaction.(current_capacity: {}, min_capacity: {})",
        input_capacity,
        storage_capacity
    );

    // User put more capacity into the AccountCell or pay the transaction directly, that will be always acceptable.
    if input_capacity > output_capacity {
        assert!(
            input_capacity - output_capacity <= fee,
            Error::AccountCellNoMoreFee,
            "The transaction fee should be less than or equal to {}, but {} found.",
            fee,
            input_capacity - output_capacity
        );
    }

    Ok(())
}

fn verify_action_throttle<'a>(
    action: &[u8],
    config: ConfigCellAccountReader,
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    current_timestamp: u64,
) -> Result<(), Error> {
    // Migration for AccountCellData v1
    macro_rules! assert_action_throttle_new {
        ($output_witness_reader:expr, ($new_field:ident, $new_field_name:expr), $( ($zero_field:ident, $zero_field_name:expr) ),*) => {{
            let current = u64::from($output_witness_reader.$new_field());
            assert!(
                current_timestamp == current,
                Error::AccountCellThrottle,
                "The AccountCell.{} in outputs should be the same as the timestamp in the TimeCell.(expected: {}, current: {})",
                $new_field_name,
                current_timestamp,
                current
            );


            $(
                let new_default_timestamp = u64::from($output_witness_reader.$zero_field());
                assert!(
                    0 == new_default_timestamp,
                    Error::AccountCellThrottle,
                    "The AccountCell.{} in outputs is new created, so it should be zero.(expected: {}, current: {})",
                    $zero_field_name,
                    0,
                    current
                );
            )*
        }}
    }

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

    let output_witness_reader = output_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;

    // Migration for AccountCellData v1
    if input_witness_reader.version() == 1 {
        match action {
            b"transfer_account" => assert_action_throttle_new!(
                output_witness_reader,
                (last_transfer_account_at, "last_transfer_account_at"),
                (last_edit_manager_at, "last_edit_manager_at"),
                (last_edit_records_at, "last_edit_records_at")
            ),
            b"edit_manager" => assert_action_throttle_new!(
                output_witness_reader,
                (last_edit_manager_at, "last_edit_manager_at"),
                (last_transfer_account_at, "last_transfer_account_at"),
                (last_edit_records_at, "last_edit_records_at")
            ),
            b"edit_records" => assert_action_throttle_new!(
                output_witness_reader,
                (last_edit_records_at, "last_edit_records_at"),
                (last_transfer_account_at, "last_transfer_account_at"),
                (last_edit_manager_at, "last_edit_manager_at")
            ),
            _ => return Err(Error::ActionNotSupported),
        }
    } else {
        let input_witness_reader = input_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

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

fn verify_records_keys<'a>(
    config: ConfigCellAccountReader,
    record_key_namespace: &Vec<u8>,
    output_account_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let output_account_witness_reader = output_account_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    let records_max_size = u32::from(config.record_size_limit()) as usize;
    let records = output_account_witness_reader.records();

    assert!(
        records.total_size() <= records_max_size,
        Error::AccountCellRecordSizeTooLarge,
        "The total size of all records can not be more than {} bytes.",
        records_max_size
    );

    // extract all the keys, which are split by 0
    let mut key_start_at = 0;
    let mut key_list = Vec::new();
    for (index, item) in record_key_namespace.iter().enumerate() {
        if *item == 0 {
            let key_vec = &record_key_namespace[key_start_at..index];
            key_start_at = index + 1;

            key_list.push(key_vec);
        }
    }

    fn vec_compare(va: &[u8], vb: &[u8]) -> bool {
        // zip stops at the shortest
        (va.len() == vb.len()) && va.iter().zip(vb).all(|(a, b)| a == b)
    }

    // check if all the record.{type+key} are valid
    for record in records.iter() {
        let mut is_valid = false;

        let mut record_type = Vec::from(record.record_type().raw_data());
        let mut record_key = Vec::from(record.record_key().raw_data());
        if record_type == b"custom_key" {
            // CAREFUL Triple check
            for char in record_key.iter() {
                assert!(
                    CUSTOM_KEYS_NAMESPACE.contains(char),
                    Error::AccountCellRecordKeyInvalid,
                    "The keys in custom_key should only contain digits, lowercase alphabet and underline."
                );
            }
            continue;
        }

        record_type.push(46);
        record_type.append(&mut record_key);

        for key in &key_list {
            if vec_compare(record_type.as_slice(), *key) {
                is_valid = true;
                break;
            }
        }

        if !is_valid {
            assert!(
                false,
                Error::AccountCellRecordKeyInvalid,
                "Account cell record key is invalid: {:?}", record_type
            );

            break;
        }
    }

    Ok(())
}
