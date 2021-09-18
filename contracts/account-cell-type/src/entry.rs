use alloc::{boxed::Box, string::String, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, error::SysError, high_level};
use das_core::{
    assert,
    constants::{das_lock, das_wallet_lock, OracleCellType, ScriptType, TypeScript, CUSTOM_KEYS_NAMESPACE},
    data_parser, debug,
    eip712::verify_eip712_hashes,
    error::Error,
    parse_account_cell_witness, parse_witness, util, verifiers, warn,
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
    let action_opt = parser.parse_action_with_params()?;

    if action_opt.is_none() {
        return Err(Error::ActionNotSupported);
    }

    let (action_raw, params_raw) = action_opt.unwrap();
    let action = action_raw.as_reader().raw_data();
    let params = params_raw.iter().map(|param| param.as_reader()).collect::<Vec<_>>();

    if action != b"init_account_chain" {
        util::is_system_off(&mut parser)?;
    }

    debug!(
        "Route to {:?} action ...",
        String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    if action == b"init_account_chain" {
        // No Root AccountCell can be created after the initialization day of DAS.
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;
        util::is_init_day(timestamp)?;

        let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) =
            util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

        assert!(
            input_cells.len() == 0,
            Error::AccountCellFoundInvalidTransaction,
            "There should be no AccountCells in inputs."
        );
        assert!(
            output_cells.len() == 1,
            Error::AccountCellFoundInvalidTransaction,
            "There should be only one AccountCells in outputs."
        );

        debug!("Check if root AccountCell uses das-lock ...");

        let index = output_cells[0];
        let expected_lock = das_lock();
        let lock_script = high_level::load_cell_lock(index, Source::Output).map_err(|e| Error::from(e))?;
        assert!(
            expected_lock.as_reader().code_hash().raw_data() == lock_script.as_reader().code_hash().raw_data(),
            Error::AccountCellFoundInvalidTransaction,
            "The lock script of AccountCell should be das-lock script."
        );
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        util::is_system_off(&mut parser)?;
        // Loading DAS witnesses and parsing the action.
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"transfer_account"
        || action == b"edit_manager"
        || action == b"edit_records"
        || action == b"sell_account"
        || action == b"cancel_account_sale"
        || action == b"buy_account"
    {
        util::is_system_off(&mut parser)?;
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;

        parser.parse_config(&[DataType::ConfigCellMain, DataType::ConfigCellAccount])?;
        parser.parse_cell()?;

        verify_eip712_hashes(&parser, action_raw.as_reader(), &params)?;

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

        assert!(
            output_cell_witness_reader.version() == 2,
            Error::DataTypeUpgradeRequired,
            "The witness of the AccountCell in outputs should be upgrade to version 2."
        );

        if action == b"sell_account" {
            debug!("Route to sell_account action ...");
            util::require_type_script(
                &mut parser,
                TypeScript::AccountSaleCellType,
                Source::Output,
                Error::AccountSaleCellMissing,
            )?;
        } else if action == b"cancel_account_sale" {
            debug!("Route to cancel_account_sale action ...");
            util::require_type_script(
                &mut parser,
                TypeScript::AccountSaleCellType,
                Source::Input,
                Error::AccountSaleCellMissing,
            )?;
        } else if action == b"buy_account" {
            debug!("Route to buy_account action ...");
            util::require_type_script(
                &mut parser,
                TypeScript::AccountSaleCellType,
                Source::Input,
                Error::AccountSaleCellMissing,
            )?;
        } else if action == b"transfer_account" {
            debug!("Route to transfer_account action ...");

            let config_account = parser.configs.account()?;

            verify_unlock_role(params_raw[0].as_reader(), LockRole::Owner)?;
            // TODO: here is complicated, should be fixed by @link
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
            verifiers::verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
            verifiers::verify_account_lock_consistent(input_account_cells[0], output_account_cells[0], Some("owner"))?;
            verifiers::verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
            verifiers::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                vec!["last_transfer_account_at"],
            )?;
        } else if action == b"edit_manager" {
            debug!("Route to edit_manager action ...");

            let config_account = parser.configs.account()?;

            verify_unlock_role(params_raw[0].as_reader(), LockRole::Owner)?;
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
            verifiers::verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
            verifiers::verify_account_lock_consistent(
                input_account_cells[0],
                output_account_cells[0],
                Some("manager"),
            )?;
            verifiers::verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
            verifiers::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                vec!["last_edit_manager_at"],
            )?;
        } else if action == b"edit_records" {
            debug!("Route to edit_records action ...");

            parser.parse_config(&[DataType::ConfigCellRecordKeyNamespace])?;
            let config_account = parser.configs.account()?;
            let record_key_namespace = parser.configs.record_key_namespace()?;

            verify_unlock_role(params_raw[0].as_reader(), LockRole::Manager)?;
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
            verifiers::verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
            verifiers::verify_account_lock_consistent(input_account_cells[0], output_account_cells[0], None)?;
            verifiers::verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
            verifiers::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                vec!["records", "last_edit_records_at"],
            )?;
            verify_records_keys(config_account, record_key_namespace, &output_cell_witness_reader)?;
        }
    } else if action == b"renew_account" {
        debug!("Route to renew_account action ...");

        util::is_system_off(&mut parser)?;

        parser.parse_cell()?;
        parser.parse_config(&[DataType::ConfigCellAccount, DataType::ConfigCellPrice])?;

        let prices = parser.configs.price()?.prices();
        let income_cell_type_id = parser.configs.main()?.type_id_table().income_cell();

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

        verify_cells_with_das_lock()?;
        verifiers::verify_account_capacity_not_decrease(input_account_cells[0], output_account_cells[0])?;
        verifiers::verify_account_lock_consistent(input_account_cells[0], output_account_cells[0], None)?;
        verifiers::verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec!["expired_at"])?;
        verifiers::verify_account_witness_consistent(
            input_account_cells[0],
            output_account_cells[0],
            &input_cell_witness_reader,
            &output_cell_witness_reader,
            vec![],
        )?;

        debug!("Check if IncomeCells in this transaction is correct.");

        let input_income_cells = util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Input)?;
        let output_income_cells = util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Output)?;

        assert!(
            input_income_cells.len() <= 1,
            Error::ProposalFoundInvalidTransaction,
            "The number of IncomeCells in inputs should be less than or equal to 1. (expected: <= 1, current: {})",
            input_income_cells.len()
        );

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
                IncomeCellData
            );

            // The IncomeCell should be a newly created cell with only one record which is belong to the creator, but we do not need to check everything here, so we only check the length.
            assert!(
                income_cell_witness_reader.records().len() == 1,
                Error::ProposalFoundInvalidTransaction,
                "The IncomeCell in inputs should be a newly created cell with only one record which is belong to the creator."
            );

            expected_first_record = income_cell_witness.records().get(0);
        }

        assert!(
            output_income_cells.len() == 1,
            Error::ProposalFoundInvalidTransaction,
            "The number of IncomeCells in outputs should be exactly 1. (expected: == 1, current: {})",
            output_income_cells.len()
        );

        let income_cell_capacity =
            high_level::load_cell_capacity(output_income_cells[0], Source::Output).map_err(|e| Error::from(e))?;
        let (_, _, entity) = parser.verify_and_get(output_income_cells[0], Source::Output)?;
        let income_cell_witness =
            IncomeCellData::from_slice(entity.as_reader().raw_data()).map_err(|_| Error::WitnessEntityDecodingError)?;
        let income_cell_witness_reader = income_cell_witness.as_reader();

        let paid;
        let das_wallet_lock = Script::from(das_wallet_lock());
        if let Some(expected_first_record) = expected_first_record {
            // IncomeCell is created before this transaction, so it is include the creator's income record.
            assert!(
                income_cell_witness_reader.records().len() == 2,
                Error::ProposalFoundInvalidTransaction,
                "The number of records of IncomeCells in outputs should be exactly 2. (expected: == 2, current: {})",
                income_cell_witness_reader.records().len()
            );

            let first_record = income_cell_witness_reader.records().get(0).unwrap();
            let exist_capacity = u64::from(first_record.capacity());

            assert!(
                util::is_reader_eq(expected_first_record.as_reader(), first_record),
                Error::ProposalFoundInvalidTransaction,
                "The first record of IncomeCell should keep the same as in inputs."
            );

            let second_record = income_cell_witness_reader.records().get(1).unwrap();
            paid = u64::from(second_record.capacity());

            assert!(
                util::is_reader_eq(second_record.belong_to(), das_wallet_lock.as_reader()),
                Error::ProposalFoundInvalidTransaction,
                "The second record in IncomeCell should belong to DAS[{}].",
                das_wallet_lock.as_reader()
            );

            assert!(
                income_cell_capacity == exist_capacity + paid,
                Error::ProposalFoundInvalidTransaction,
                "The capacity of IncomeCell in outputs is incorrect. (expected: {}, current: {})",
                exist_capacity + paid,
                income_cell_capacity
            );
        } else {
            // IncomeCell is created with only profit.
            assert!(
                income_cell_witness_reader.records().len() == 1,
                Error::ProposalFoundInvalidTransaction,
                "The number of records of IncomeCells in outputs should be exactly 2. (expected: == 2, current: {})",
                income_cell_witness_reader.records().len()
            );

            let first_record = income_cell_witness_reader.records().get(0).unwrap();
            paid = u64::from(first_record.capacity());

            assert!(
                util::is_reader_eq(first_record.belong_to(), das_wallet_lock.as_reader()),
                Error::ProposalFoundInvalidTransaction,
                "The only record in IncomeCell should belong to DAS[{}].",
                das_wallet_lock.as_reader()
            );

            assert!(
                income_cell_capacity == paid,
                Error::ProposalFoundInvalidTransaction,
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
            "The AccountCell renew should be longer than 1 year. current({}) < expected(31_536_000)",
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

        // The AccountCell can be used as long as it is not modified.
    } else if action == b"recycle_expired_account_by_keeper" {
        debug!("Route to recycle_expired_account_by_keeper action ...");
        return Err(Error::InvalidTransactionStructure);

        util::is_system_off(&mut parser)?;
        let timestamp = util::load_oracle_data(OracleCellType::Time)?;

        parser.parse_cell()?;
        parser.parse_config(&[DataType::ConfigCellAccount])?;

        verify_cells_with_das_lock()?;

        let config_account = parser.configs.account()?;

        // The AccountCell should be recycled in the transaction.
        let (input_account_cells, output_account_cells) = load_account_cells()?;
        assert!(
            input_account_cells.len() == 1 && output_account_cells.len() == 0,
            Error::AccountCellFoundInvalidTransaction,
            "There should be 1 AccountCell in inputs and none in outputs."
        );

        debug!("Check if account has reached the end off the expiration grace period.");

        let expiration_grace_period = u32::from(config_account.expiration_grace_period()) as u64;
        let account_data = util::load_cell_data(input_account_cells[0], Source::Input)?;
        let expired_at = data_parser::account_cell::get_expired_at(&account_data);

        assert!(
            expired_at + expiration_grace_period < timestamp,
            Error::AccountCellIsNotExpired,
            "The recovery of the account should be executed after the grace period. (current({}) <= expired_at({}) + grace_period({}))",
            timestamp,
            expired_at,
            expiration_grace_period
        );
    } else {
        debug!("Route to other action ...");
        // TODO Stop unknown transaction occupy AccountCells.

        let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) =
            util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

        assert!(
            input_cells.len() == output_cells.len(),
            Error::CellsMustHaveSameOrderAndNumber,
            "The AccountCells in inputs should have the same number and order as those in outputs."
        );

        util::is_inputs_and_outputs_consistent(input_cells, output_cells)?;
    }

    Ok(())
}

fn verify_input_account_must_normal_status<'a>(
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Error> {
    let witness_reader = input_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    let account_status = u8::from(witness_reader.status());
    if account_status != (AccountStatus::Normal as u8) {
        return Err(Error::AccountCellSaleStatusNotAllow);
    }
    return Ok(());
}

fn load_account_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let (input_account_cells, output_account_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_type_script.as_reader())?;

    Ok((input_account_cells, output_account_cells))
}

fn verify_unlock_role(params: BytesReader, lock: LockRole) -> Result<(), Error> {
    debug!("Check if transaction is unlocked by {:?}.", lock);

    assert!(
        params.len() > 0 && params.raw_data()[0] == lock as u8,
        Error::AccountCellPermissionDenied,
        "This transaction should be unlocked by the {:?}'s signature.",
        lock
    );

    Ok(())
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

fn verify_cells_with_das_lock() -> Result<(), Error> {
    let this_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let this_script_reader = this_script.as_reader();

    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();
    let mut i = 0;
    loop {
        let ret = high_level::load_cell_lock(i, Source::Input);
        match ret {
            Ok(lock) => {
                // Check if cells with das-lock in inputs can only has account-cell-type.
                if util::is_script_equal(das_lock_reader, lock.as_reader()) {
                    let type_opt = high_level::load_cell_type(i, Source::Input).map_err(|e| Error::from(e))?;
                    match type_opt {
                        Some(type_) if util::is_reader_eq(this_script_reader, type_.as_reader()) => {}
                        _ => {
                            warn!(
                                "Inputs[{}] This cell has das-lock, normal cells with das-lock is not allowed in this transaction.",
                                i
                            );
                            return Err(Error::InvalidTransactionStructure);
                        }
                    }
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

    Ok(())
}
