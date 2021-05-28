use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    debug,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock, load_script},
};
use das_core::{
    assert,
    constants::{das_lock, das_wallet_lock, super_lock, ScriptType, TypeScript},
    data_parser,
    error::Error,
    util, warn,
    witness_parser::WitnessesParser,
};
use das_types::{
    constants::{DataType, LockRole},
    packed::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-cell-type ======");

    let mut parser = WitnessesParser::new()?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    let params = action_data.as_reader().params().raw_data();
    if action == b"init_account_chain" {
        debug!("Route to init_account_chain action ...");

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) = util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;

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

        debug!("Check if super lock has been used in inputs ...");

        let super_lock = super_lock();
        let has_super_lock =
            util::find_cells_by_script(ScriptType::Lock, super_lock.as_reader(), Source::Input)?
                .len()
                > 0;

        assert!(
            has_super_lock,
            Error::SuperLockIsRequired,
            "The super lock is required."
        );

        debug!("Check if root AccountCell uses das-lock ...");

        let index = output_cells[0];
        let expected_lock = das_lock();
        let lock_script = load_cell_lock(index, Source::Output).map_err(|e| Error::from(e))?;
        assert!(
            expected_lock.as_reader().code_hash().raw_data()
                == lock_script.as_reader().code_hash().raw_data(),
            Error::AccountCellFoundInvalidTransaction,
            "The lock script of AccountCell should be das-lock script."
        );
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        // Loading DAS witnesses and parsing the action.
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"transfer_account" {
        debug!("Route to transfer_account action ...");

        let timestamp = util::load_timestamp()?;

        parser.parse_config(&[DataType::ConfigCellAccount])?;
        parser.parse_cell()?;

        let config_account = parser.configs.account()?;
        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_unlock_role(params, LockRole::Owner)?;
        verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
        verify_account_consistent(
            input_account_cells[0],
            output_account_cells[0],
            Some("owner"),
        )?;
        verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
        verify_account_witness_consistent(
            &parser,
            input_account_cells[0],
            output_account_cells[0],
            vec![],
        )?;
    } else if action == b"edit_manager" {
        debug!("Route to edit_manager action ...");

        let timestamp = util::load_timestamp()?;

        parser.parse_config(&[DataType::ConfigCellAccount])?;
        parser.parse_cell()?;

        let config_account = parser.configs.account()?;
        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_unlock_role(params, LockRole::Owner)?;
        verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
        verify_account_consistent(
            input_account_cells[0],
            output_account_cells[0],
            Some("manager"),
        )?;
        verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
        verify_account_witness_consistent(
            &parser,
            input_account_cells[0],
            output_account_cells[0],
            vec![],
        )?;
    } else if action == b"edit_records" {
        debug!("Route to edit_records action ...");

        let timestamp = util::load_timestamp()?;

        parser.parse_cell()?;
        parser.parse_config(&[
            DataType::ConfigCellAccount,
            DataType::ConfigCellMain,
            DataType::ConfigCellRecordKeyNamespace,
        ])?;

        let config_account = parser.configs.account()?;
        let record_key_namespace = parser.configs.record_key_namespace()?;
        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_unlock_role(params, LockRole::Manager)?;
        verify_account_expiration(config_account, input_account_cells[0], timestamp)?;
        verify_account_consistent(input_account_cells[0], output_account_cells[0], None)?;
        verify_account_data_consistent(input_account_cells[0], output_account_cells[0], vec![])?;
        verify_account_witness_consistent(
            &parser,
            input_account_cells[0],
            output_account_cells[0],
            vec!["records"],
        )?;
        verify_records_keys(&parser, record_key_namespace, output_account_cells[0])?;
    } else if action == b"renew_account" {
        debug!("Route to renew_account action ...");

        parser.parse_cell()?;
        parser.parse_config(&[
            DataType::ConfigCellAccount,
            DataType::ConfigCellMain,
            DataType::ConfigCellPrice,
        ])?;

        // let expiration_grace_period =
        //     u32::from(parser.configs.account()?.expiration_grace_period());
        let prices = parser.configs.price()?.prices();
        let income_cell_type_id = parser.configs.main()?.type_id_table().income_cell();

        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_account_consistent(input_account_cells[0], output_account_cells[0], None)?;
        verify_account_data_consistent(
            input_account_cells[0],
            output_account_cells[0],
            vec!["expired_at"],
        )?;
        verify_account_witness_consistent(
            &parser,
            input_account_cells[0],
            output_account_cells[0],
            vec![],
        )?;

        debug!("Check if IncomeCells in this transaction is correct.");

        let input_income_cells =
            util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Input)?;
        let output_income_cells =
            util::find_cells_by_type_id(ScriptType::Type, income_cell_type_id, Source::Output)?;

        assert!(
            input_income_cells.len() <= 1,
            Error::ProposalFoundInvalidTransaction,
            "The number of IncomeCells in inputs should be less than or equal to 1. (expected: <= 1, current: {})",
            input_income_cells.len()
        );

        let mut expected_first_record = None;
        if input_income_cells.len() == 1 {
            let (_, _, entity) = parser.verify_and_get(input_income_cells[0], Source::Input)?;
            let income_cell_witness = IncomeCellData::from_slice(entity.as_reader().raw_data())
                .map_err(|_| Error::WitnessEntityDecodingError)?;
            let income_cell_witness_reader = income_cell_witness.as_reader();

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

        let income_cell_capacity = load_cell_capacity(output_income_cells[0], Source::Output)
            .map_err(|e| Error::from(e))?;
        let (_, _, entity) = parser.verify_and_get(output_income_cells[0], Source::Output)?;
        let income_cell_witness = IncomeCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
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
            let storage_capacity = u64::from(first_record.capacity());

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
                income_cell_capacity == storage_capacity + paid,
                Error::ProposalFoundInvalidTransaction,
                "The capacity of IncomeCell in outputs is incorrect. (expected: {}, current: {})",
                storage_capacity + paid,
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

        let (_, _, entity) = parser.verify_and_get(input_account_cells[0], Source::Input)?;
        let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let input_witness_reader = input_account_witness.as_reader();

        let length_in_price =
            util::get_length_in_price(input_witness_reader.account().len() as u64);

        // Find out renew price in USD.
        let mut price_opt = Some(prices.get(prices.len() - 1).unwrap());
        for item in prices.iter() {
            if u8::from(item.length()) == length_in_price {
                price_opt = Some(item);
                break;
            }
        }
        let renew_price_in_usd = u64::from(price_opt.unwrap().renew()); // x USD
        let quote = util::load_quote()?;

        // Renew price for 1 year in CKB = x ÷ y .
        let expected_duration = util::calc_duration_from_paid(paid, renew_price_in_usd, quote, 0);
        if duration > expected_duration {
            debug!(
                "Verify is user payed enough capacity: duration({}) > (paid({}) / (renew_price({}) / quote({}) * 100_000_000) ) * 86400 * 365 -> true",
                duration,
                paid,
                renew_price_in_usd,
                quote
            );

            return Err(Error::AccountCellRenewDurationBiggerThanPaied);
        }

        // The AccountCell can be used as long as it is not modified.
    } else if action == b"recycle_expired_account_by_keeper" {
        debug!("Route to recycle_expired_account_by_keeper action ...");

        let timestamp = util::load_timestamp()?;

        parser.parse_cell()?;
        parser.parse_config(&[DataType::ConfigCellAccount])?;

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
        if expired_at + expiration_grace_period >= timestamp {
            return Err(Error::AccountCellIsNotExpired);
        }

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

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) = util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;

        assert!(
            input_cells.len() == output_cells.len(),
            Error::CellsMustHaveSameOrderAndNumber,
            "The AccountCells in inputs should have the same number and order as those in outputs."
        );

        util::is_inputs_and_outputs_consistent(input_cells, output_cells)?;
    }

    Ok(())
}

fn load_account_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let (input_account_cells, output_account_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;

    Ok((input_account_cells, output_account_cells))
}

fn verify_unlock_role(params: &[u8], lock: LockRole) -> Result<(), Error> {
    debug!("Check if transaction is unlocked by {:?}.", lock);

    assert!(
        params.len() > 0 && params[0] == lock as u8,
        Error::AccountCellPermissionDenied,
        "This transaction should be unlocked by the {:?}'s signature.",
        lock
    );

    Ok(())
}

fn verify_account_consistent(
    input_account_index: usize,
    output_account_index: usize,
    changed_lock: Option<&str>,
) -> Result<(), Error> {
    debug!("Check if everything consistent except data in the AccountCell.");

    util::is_cell_capacity_equal(
        (input_account_index, Source::Input),
        (output_account_index, Source::Output),
    )?;
    util::is_cell_type_equal(
        (input_account_index, Source::Input),
        (output_account_index, Source::Output),
    )?;

    if let Some(lock) = changed_lock {
        let input_lock =
            load_cell_lock(input_account_index, Source::Input).map_err(|e| Error::from(e))?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock =
            load_cell_lock(output_account_index, Source::Output).map_err(|e| Error::from(e))?;
        let output_args = output_lock.as_reader().args().raw_data();

        if lock == "owner" {
            assert!(
                data_parser::das_lock_args::get_owner_lock_args(input_args)
                    != data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellOwnerLockShouldBeModified,
                "The owner lock args in AccountCell.lock should be different in input and output."
            );
        } else {
            assert!(
                data_parser::das_lock_args::get_owner_lock_args(input_args)
                    == data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellOwnerLockShouldNotBeModified,
                "The owner lock args in AccountCell.lock should be consistent in input and output."
            );

            assert!(
                data_parser::das_lock_args::get_manager_lock_args(input_args)
                    != data_parser::das_lock_args::get_manager_lock_args(output_args),
                Error::AccountCellManagerLockShouldBeModified,
                "The manager lock args in AccountCell.lock should be different in input and output."
            );
        }
    } else {
        util::is_cell_lock_equal(
            (input_account_index, Source::Input),
            (output_account_index, Source::Output),
        )?;
    }

    Ok(())
}

fn verify_account_data_consistent(
    input_account_index: usize,
    output_account_index: usize,
    except: Vec<&str>,
) -> Result<(), Error> {
    debug!("Check if AccountCell.data is consistent in input and output.");

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let output_data = util::load_cell_data(output_account_index, Source::Output)?;

    assert!(
        data_parser::account_cell::get_id(&input_data)
            == data_parser::account_cell::get_id(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.id field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        data_parser::account_cell::get_next(&input_data)
            == data_parser::account_cell::get_next(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.next field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        data_parser::account_cell::get_account(&input_data)
            == data_parser::account_cell::get_account(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.account field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    if !except.contains(&"expired_at") {
        assert!(
            data_parser::account_cell::get_expired_at(&input_data)
                == data_parser::account_cell::get_expired_at(&output_data),
            Error::AccountCellDataNotConsistent,
            "The data.expired_at field of inputs[{}] and outputs[{}] should be the same.",
            input_account_index,
            output_account_index
        );
    }

    Ok(())
}

fn verify_account_expiration(
    config: ConfigCellAccountReader,
    account_cell_index: usize,
    current: u64,
) -> Result<(), Error> {
    debug!("Check if AccountCell is expired.");

    let data = load_cell_data(account_cell_index, Source::Input).map_err(|e| Error::from(e))?;
    let expired_at = data_parser::account_cell::get_expired_at(data.as_slice());
    let expiration_grace_period = u32::from(config.expiration_grace_period()) as u64;

    if current > expired_at {
        if current - expired_at > expiration_grace_period {
            warn!("The AccountCell has been expired. Will be recycled soon.");
            return Err(Error::AccountCellHasExpired);
        } else {
            warn!("The AccountCell has been in expiration grace period. Need to be renew as soon as possible.");
            return Err(Error::AccountCellInExpirationGracePeriod);
        }
    }

    Ok(())
}

fn verify_account_witness_consistent(
    parser: &WitnessesParser,
    input_account_index: usize,
    output_account_index: usize,
    except: Vec<&str>,
) -> Result<(), Error> {
    debug!("Check if AccountCell.witness is consistent in input and output.");

    let (_, _, entity) = parser.verify_and_get(input_account_index, Source::Input)?;
    let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
        .map_err(|_| Error::WitnessEntityDecodingError)?;
    let input_witness_reader = input_account_witness.as_reader();
    let (_, _, entity) = parser.verify_and_get(output_account_index, Source::Output)?;
    let output_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
        .map_err(|_| Error::WitnessEntityDecodingError)?;
    let output_witness_reader = output_account_witness.as_reader();

    assert!(
        util::is_reader_eq(input_witness_reader.id(), output_witness_reader.id()),
        Error::AccountCellProtectFieldIsModified,
        "The witness.id field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        util::is_reader_eq(
            input_witness_reader.account(),
            output_witness_reader.account()
        ),
        Error::AccountCellProtectFieldIsModified,
        "The witness.account field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        util::is_reader_eq(
            input_witness_reader.registered_at(),
            output_witness_reader.registered_at()
        ),
        Error::AccountCellProtectFieldIsModified,
        "The witness.registered_at field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        util::is_reader_eq(
            input_witness_reader.status(),
            output_witness_reader.status()
        ),
        Error::AccountCellProtectFieldIsModified,
        "The witness.status field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );

    if !except.contains(&"records") {
        assert!(
            util::is_reader_eq(
                input_witness_reader.records(),
                output_witness_reader.records()
            ),
            Error::AccountCellProtectFieldIsModified,
            "The witness.records field of inputs[{}] and outputs[{}] should be the same.",
            input_account_index,
            output_account_index
        );
    }

    Ok(())
}

fn verify_records_keys(
    parser: &WitnessesParser,
    record_key_namespace: &Vec<u8>,
    output_account_index: usize,
) -> Result<(), Error> {
    let (_, _, entity) = parser.verify_and_get(output_account_index, Source::Output)?;
    let output_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data()).map_err(|_| Error::WitnessEntityDecodingError)?;
    let records = output_account_witness.as_reader().records();

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
        (va.len() == vb.len()) &&
            va.iter()
                .zip(vb)
                .all(|(a,b)| a == b)
    }

    // check if all the record.{type+key} are valid
    for record in records.iter() {
        let mut is_valid = false;

        let mut record_type = Vec::from(record.record_type().raw_data());
        let mut record_key = Vec::from(record.record_key().raw_data());
        record_type.push(46);
        record_type.append(&mut record_key);

        for key in &key_list {
            if vec_compare(record_type.as_slice(), *key) {
                is_valid = true;
                break
            }
        }

        if !is_valid {
            assert!(
                false,
                Error::AccountCellRecordKeyInvalid,
                "Account cell record key is invalid: {:?}",
                record_type
            );

            break;
        }
    }

    Ok(())
}
