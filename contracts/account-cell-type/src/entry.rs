use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    debug, high_level,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_script,
    },
};
use das_core::{
    assert,
    constants::{super_lock, ScriptType, TypeScript, ALWAYS_SUCCESS_LOCK, DAS_WALLET_ID},
    data_parser,
    data_parser::account_cell,
    error::Error,
    util, warn,
    witness_parser::WitnessesParser,
};
use das_types::{
    constants::{ConfigID, DataType},
    packed::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-cell-type ======");

    let action_data = util::load_das_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"init_account_chain" {
        debug!("Route to init_account_chain action ...");

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let input_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let output_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

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
            util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len() > 0;

        assert!(
            has_super_lock,
            Error::SuperLockIsRequired,
            "The super lock is required."
        );

        debug!("Check if root AccountCell uses always_success lock ...");

        let index = output_cells[0];
        let always_success_script = util::script_literal_to_script(ALWAYS_SUCCESS_LOCK);
        let always_success_script_hash = util::blake2b_256(always_success_script.as_slice());
        let lock_script = load_cell_lock_hash(index, Source::Output).map_err(|e| Error::from(e))?;
        if lock_script != always_success_script_hash {
            return Err(Error::WalletRequireAlwaysSuccess);
        }
        assert!(
            lock_script == always_success_script_hash,
            Error::AccountCellFoundInvalidTransaction,
            "The lock script of AccountCell should be always-success script."
        );
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        // Loading DAS witnesses and parsing the action.
        let mut parser = util::load_das_witnesses(Some(vec![DataType::ConfigCellMain]))?;
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"transfer_account" {
        debug!("Route to transfer_account action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config = parser.configs().main()?;
        let timestamp = util::load_timestamp()?;

        let input_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Input,
        )?;
        let output_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Output,
        )?;
        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_account_expiration(input_account_cells[0], timestamp)?;
        verify_account_consistent(input_account_cells[0], output_account_cells[0])?;
        verify_account_data_consistent(input_account_cells[0], output_account_cells[0])?;

        debug!("Check the relationship between RefCells and AccountCell is correct.");

        // This will ensure that RefCells in inputs and outputs is unique and referenced by AccountCell.
        let (input_owner_cell, input_manager_cell) = distinguish_owner_and_manager(
            &parser,
            input_account_cells[0],
            input_ref_cells,
            Source::Input,
        )?;
        let (output_owner_cell, output_manager_cell) = distinguish_owner_and_manager(
            &parser,
            output_account_cells[0],
            output_ref_cells,
            Source::Output,
        )?;

        assert!(
            input_owner_cell.is_some() && output_owner_cell.is_some(),
            Error::AccountCellOwnerCellIsRequired,
            "The OwnerCell should exist in both inputs and outputs."
        );

        assert!(
            input_manager_cell.is_none() && output_manager_cell.is_none(),
            Error::AccountCellRedundantRefCellNotAllowed,
            "The ManagerCell should not exist in either inputs or outputs."
        );

        util::is_cell_only_lock_changed(
            (input_owner_cell.unwrap(), Source::Input),
            (output_owner_cell.unwrap(), Source::Output),
        )?;

        debug!(
            "Check if every fields except owner_lock and manager_lock in witness are consistent."
        );

        let (_, _, entity) = parser.verify_and_get(input_account_cells[0], Source::Input)?;
        let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let input_witness_reader = input_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(output_account_cells[0], Source::Output)?;
        let output_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let output_witness_reader = output_account_witness.as_reader();

        verify_if_id_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_account_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_registered_at_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_status_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_records_consistent(input_witness_reader, output_witness_reader)?;
    } else if action == b"edit_manager" {
        debug!("Route to edit_manager action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let timestamp = util::load_timestamp()?;

        let config = parser.configs().main()?;

        let input_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Input,
        )?;
        let output_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Output,
        )?;
        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_account_expiration(input_account_cells[0], timestamp)?;
        verify_account_consistent(input_account_cells[0], output_account_cells[0])?;
        verify_account_data_consistent(input_account_cells[0], output_account_cells[0])?;

        debug!("Check the relationship between RefCells and AccountCell is correct.");

        // This will ensure that RefCells in inputs and outputs is unique and referenced by AccountCell.
        let (input_owner_cell, input_manager_cell) = distinguish_owner_and_manager(
            &parser,
            input_account_cells[0],
            input_ref_cells,
            Source::Input,
        )?;
        let (output_owner_cell, output_manager_cell) = distinguish_owner_and_manager(
            &parser,
            output_account_cells[0],
            output_ref_cells,
            Source::Output,
        )?;

        assert!(
            input_owner_cell.is_some() && output_owner_cell.is_some(),
            Error::AccountCellManagerCellIsRequired,
            "The OwnerCell is required in both inputs and outputs."
        );

        assert!(
            output_manager_cell.is_some(),
            Error::AccountCellManagerCellIsRequired,
            "The ManagerCell is required in outputs."
        );

        if input_manager_cell.is_some() {
            util::is_cell_only_lock_changed(
                (input_manager_cell.unwrap(), Source::Input),
                (output_manager_cell.unwrap(), Source::Output),
            )?;
        } else {
            let account_cell_data =
                util::load_cell_data(output_owner_cell.unwrap(), Source::Output)?;
            let manager_cell_data =
                util::load_cell_data(output_manager_cell.unwrap(), Source::Output)?;

            let account_cell_data_id = data_parser::ref_cell::get_id(&account_cell_data);
            let manager_cell_data_id = data_parser::ref_cell::get_id(&manager_cell_data);

            assert!(
                account_cell_data_id == manager_cell_data_id,
                Error::AccountCellFoundInvalidTransaction,
                "The data.id of new ManagerCell should be the same as the AccountCell."
            );

            let manager_cell_data_is_owner =
                data_parser::ref_cell::get_is_owner(&manager_cell_data);
            assert!(
                !manager_cell_data_is_owner,
                Error::AccountCellFoundInvalidTransaction,
                "The data.is_owner of new ManagerCell should be 0x01."
            );
        }

        debug!("Check if every fields except manager_lock in witness are consistent.");

        let (_, _, entity) = parser.verify_and_get(input_account_cells[0], Source::Input)?;
        let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let input_witness_reader = input_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(output_account_cells[0], Source::Output)?;
        let output_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let output_witness_reader = output_account_witness.as_reader();

        verify_if_id_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_account_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_owner_lock_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_registered_at_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_status_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_records_consistent(input_witness_reader, output_witness_reader)?;
    } else if action == b"edit_records" {
        debug!("Route to edit_records action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;

        let config = parser.configs().main()?;
        let timestamp = util::load_timestamp()?;

        let input_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Input,
        )?;
        let output_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Output,
        )?;
        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_account_expiration(input_account_cells[0], timestamp)?;
        verify_account_consistent(input_account_cells[0], output_account_cells[0])?;
        verify_account_data_consistent(input_account_cells[0], output_account_cells[0])?;

        debug!("Check the relationship between RefCells and AccountCell is correct.");

        // This will ensure that RefCells in inputs and outputs is unique and referenced by AccountCell.
        let (input_owner_cell, input_manager_cell) = distinguish_owner_and_manager(
            &parser,
            input_account_cells[0],
            input_ref_cells,
            Source::Input,
        )?;
        let (output_owner_cell, output_manager_cell) = distinguish_owner_and_manager(
            &parser,
            output_account_cells[0],
            output_ref_cells,
            Source::Output,
        )?;
        // Check if OwnerCell exists in inputs and outputs.
        if input_owner_cell.is_some() || output_owner_cell.is_some() {
            return Err(Error::AccountCellRedundantRefCellNotAllowed);
        }
        // Check if ManagerCell not exists in inputs and outputs.
        if input_manager_cell.is_none() || output_manager_cell.is_none() {
            return Err(Error::AccountCellManagerCellIsRequired);
        }

        assert!(
            input_owner_cell.is_none() && output_owner_cell.is_none(),
            Error::AccountCellRedundantRefCellNotAllowed,
            "The OwnerCell should not exist in either inputs or outputs."
        );

        assert!(
            input_manager_cell.is_some() && output_manager_cell.is_some(),
            Error::AccountCellManagerCellIsRequired,
            "The ManagerCell should exist in both inputs and outputs."
        );

        debug!("Check if every fields except records in witness are consistent.");

        let (_, _, entity) = parser.verify_and_get(input_account_cells[0], Source::Input)?;
        let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let input_witness_reader = input_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(output_account_cells[0], Source::Output)?;
        let output_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let output_witness_reader = output_account_witness.as_reader();

        verify_if_id_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_account_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_owner_lock_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_manager_lock_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_registered_at_consistent(input_witness_reader, output_witness_reader)?;
        verify_if_status_consistent(input_witness_reader, output_witness_reader)?;
    } else if action == b"renew_account" {
        debug!("Route to reoutput_account action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain, ConfigID::ConfigCellRegister])?;

        let config_main = parser.configs().main()?;
        let config_register = parser.configs().register()?;

        let (input_account_cells, output_account_cells) = load_account_cells()?;

        verify_account_consistent(input_account_cells[0], output_account_cells[0])?;
        verify_account_data_except_expired_at_consistent(
            input_account_cells[0],
            output_account_cells[0],
        )?;

        debug!("Check if the renewal duration is longer than or equal to one year.");

        let input_data = util::load_cell_data(input_account_cells[0], Source::Input)?;
        let output_data = util::load_cell_data(output_account_cells[0], Source::Output)?;
        let input_expired_at = account_cell::get_expired_at(&input_data);
        let output_expired_at = account_cell::get_expired_at(&output_data);
        let duration = output_expired_at - input_expired_at;

        assert!(
            duration >= 365 * 86400,
            Error::AccountCellRenewDurationMustLongerThanYear,
            "The AccountCell renew should be longer than 1 year. current({}) < expected(31_536_000)",
            duration
        );

        debug!("Check if the registered_at field has been updated correctly based on the capacity paid by the user.");

        let (_, _, entity) = parser.verify_and_get(input_account_cells[0], Source::Input)?;
        let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let input_witness_reader = input_account_witness.as_reader();

        let length_in_price =
            util::get_length_in_price(input_witness_reader.account().len() as u64);
        let prices = config_register.price_configs();

        // Find out renew price in USD.
        let mut price_opt = Some(prices.get(prices.len() - 1).unwrap());
        for item in prices.iter() {
            if u8::from(item.length()) == length_in_price {
                price_opt = Some(item);
                break;
            }
        }
        let renew_price_in_usd = u64::from(price_opt.unwrap().renew()); // x USD

        // Find out all WalletCells in transaction.
        let (input_wallet_cells, output_wallet_cells) = load_wallet_cells(config_main)?;

        assert!(
            input_wallet_cells.len() == 1 && output_wallet_cells.len() == 1,
            Error::AccountCellFoundInvalidTransaction,
            "There should be a WalletCell exist in both inputs and outputs."
        );

        let quote = util::load_quote()?;

        let input_wallet_capacity =
            load_cell_capacity(input_wallet_cells[0], Source::Input).map_err(|e| Error::from(e))?;
        let output_wallet_capacity = load_cell_capacity(output_wallet_cells[0], Source::Output)
            .map_err(|e| Error::from(e))?;

        // Renew price for 1 year in CKB = x ÷ y .
        let renew_price = renew_price_in_usd / quote * 100_000_000;

        let expected_duration =
            (output_wallet_capacity - input_wallet_capacity) * 86400 * 365 / renew_price;
        if duration > expected_duration {
            debug!("Verify is user payed enough capacity: {}[duration] > ({}[after_ckb] - {}[before_ckb]) * 86400 * 365 / {}[renew_price] -> true",
                   duration,
                   output_wallet_capacity,
                   input_wallet_capacity,
                   renew_price
            );

            return Err(Error::AccountCellRenewDurationBiggerThanPaied);
        }

        // The AccountCell can be used as long as it is not modified.
    } else if action == b"recycle_expired_account_by_keeper" {
        debug!("Route to recycle_expired_account_by_keeper action ...");

        let timestamp = util::load_timestamp()?;

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;

        let config_main = parser.configs().main()?;

        // The AccountCell should be recycled in the transaction.
        let (input_account_cells, output_account_cells) = load_account_cells()?;
        if input_account_cells.len() != 1 || output_account_cells.len() != 0 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        debug!("Check if account has reached the end off the expiration grace period.");

        let expiration_grace_period =
            u32::from(config_main.account_expiration_grace_period()) as u64;
        let account_data = util::load_cell_data(input_account_cells[0], Source::Input)?;
        let expired_at = account_cell::get_expired_at(&account_data);
        if expired_at + expiration_grace_period >= timestamp {
            return Err(Error::AccountCellIsNotExpired);
        }

        let account_id = account_cell::get_id(&account_data);

        debug!("Check if the transaction has required WalletCells.");

        let (input_wallet_cells, output_wallet_cells) = load_wallet_cells(config_main)?;

        // There should be a WalletCell of the account and a WalletCell of DAS in inputs.
        let mut account_wallet = None;
        let mut input_das_wallet = None;
        for index in input_wallet_cells {
            let type_script = high_level::load_cell_type(index, Source::Input)
                .map_err(|e| Error::from(e))?
                .unwrap();
            let id = type_script.as_reader().args().raw_data();
            if id == account_id {
                account_wallet = Some(index);
            } else if id == &DAS_WALLET_ID {
                input_das_wallet = Some(index);
            } else {
                return Err(Error::AccountCellFoundInvalidTransaction);
            }
        }

        // The WalletCell of the account should be recycled either.
        if output_wallet_cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        let type_script = high_level::load_cell_type(output_wallet_cells[0], Source::Input)
            .map_err(|e| Error::from(e))?
            .unwrap();
        let id = type_script.as_reader().args().raw_data();
        if id == &DAS_WALLET_ID {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }
        let new_das_wallet = output_wallet_cells[0];

        debug!("Check if the DAS WalletCell's balance has increased correctly.");

        let account_wallet_occupied_capacity =
            high_level::load_cell_occupied_capacity(account_wallet.unwrap(), Source::Input)
                .map_err(|e| Error::from(e))?;
        let input_capacity =
            high_level::load_cell_capacity(input_das_wallet.unwrap(), Source::Input)
                .map_err(|e| Error::from(e))?;
        let output_capacity = high_level::load_cell_capacity(new_das_wallet, Source::Output)
            .map_err(|e| Error::from(e))?;
        if output_capacity - input_capacity < account_wallet_occupied_capacity {
            debug!(
                "Compare recycle capacity: {}[output_capacity] - {}[input_capacity] < {}[account_wallet_occupied_capacity] => true",
                output_capacity,
                input_capacity,
                account_wallet_occupied_capacity
            );
            return Err(Error::AccountCellRecycleCapacityError);
        }

        debug!("Check if the User's owner lock get correct change.");

        let (_, _, entity) = parser.verify_and_get(input_account_cells[0], Source::Input)?;
        let owner_lock = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?
            .owner_lock();
        let cells =
            util::find_cells_by_script(ScriptType::Lock, &owner_lock.into(), Source::Output)?;

        if cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        let account_cell_capacity =
            high_level::load_cell_capacity(input_account_cells[0], Source::Input)
                .map_err(|e| Error::from(e))?;
        let account_wallet_capacity =
            high_level::load_cell_capacity(account_wallet.unwrap(), Source::Input)
                .map_err(|e| Error::from(e))?;
        let expected_change =
            account_cell_capacity + account_wallet_capacity - account_wallet_occupied_capacity;
        let change_capacity =
            high_level::load_cell_capacity(cells[0], Source::Output).map_err(|e| Error::from(e))?;

        if expected_change > change_capacity {
            debug!(
                "Compare change capacity: {}[account_cell_capacity] + {}[account_wallet_capacity] - {}[account_wallet_occupied_capacity] > {}[change_capacity] => true",
                account_cell_capacity,
                account_wallet_capacity,
                account_wallet_occupied_capacity,
                change_capacity
            );
            return Err(Error::AccountCellChangeCapacityError);
        }
    } else {
        debug!("Route to other action ...");

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) =
            util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, &this_type_script)?;

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
    let input_account_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let output_account_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

    Ok((input_account_cells, output_account_cells))
}

fn load_wallet_cells(config: ConfigCellMainReader) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let input_wallet_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        config.type_id_table().wallet_cell(),
        Source::Input,
    )?;
    let output_wallet_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        config.type_id_table().wallet_cell(),
        Source::Output,
    )?;

    Ok((input_wallet_cells, output_wallet_cells))
}

fn verify_account_consistent(
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if everything consistent except data in the AccountCell.");

    util::is_cell_capacity_equal(
        (input_account_index, Source::Input),
        (output_account_index, Source::Output),
    )?;
    util::is_cell_lock_equal(
        (input_account_index, Source::Input),
        (output_account_index, Source::Output),
    )?;
    util::is_cell_type_equal(
        (input_account_index, Source::Input),
        (output_account_index, Source::Output),
    )?;

    Ok(())
}

fn verify_account_data_consistent(
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if data consistent in the AccountCell.");

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let output_data = util::load_cell_data(output_account_index, Source::Output)?;

    assert!(
        account_cell::get_id(&input_data) == account_cell::get_id(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.id of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        account_cell::get_next(&input_data) == account_cell::get_next(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.next of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        account_cell::get_account(&input_data) == account_cell::get_account(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.account of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        account_cell::get_expired_at(&input_data) == account_cell::get_expired_at(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.expired_at of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );

    Ok(())
}

fn verify_account_data_except_expired_at_consistent(
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if data consistent in the AccountCell.");

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let output_data = util::load_cell_data(output_account_index, Source::Output)?;

    assert!(
        account_cell::get_id(&input_data) == account_cell::get_id(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.id of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        account_cell::get_next(&input_data) == account_cell::get_next(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.next of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        account_cell::get_account(&input_data) == account_cell::get_account(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.account of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        input_data.get(..32) == output_data.get(..32),
        Error::AccountCellDataNotConsistent,
        "The data.hash of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );

    Ok(())
}

fn verify_account_expiration(account_cell_index: usize, current: u64) -> Result<(), Error> {
    debug!("Check if AccountCell is expired.");

    let data = load_cell_data(account_cell_index, Source::Input).map_err(|e| Error::from(e))?;
    let expired_at = account_cell::get_expired_at(data.as_slice());

    if current > expired_at {
        if current - expired_at > 86400 * 30 {
            warn!("The AccountCell has been expired. Will be recycled soon.");
            return Err(Error::AccountCellHasExpired);
        } else {
            warn!("The AccountCell has been in expiration grace period. Need to be renew as soon as possible.");
            return Err(Error::AccountCellInExpirationGracePeriod);
        }
    }

    Ok(())
}

fn verify_if_id_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(input_witness_reader.id(), output_witness_reader.id()) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_owner_lock_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        input_witness_reader.owner_lock(),
        output_witness_reader.owner_lock(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_manager_lock_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        input_witness_reader.manager_lock(),
        output_witness_reader.manager_lock(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_account_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        input_witness_reader.account(),
        output_witness_reader.account(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_registered_at_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        input_witness_reader.registered_at(),
        output_witness_reader.registered_at(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_status_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        input_witness_reader.status(),
        output_witness_reader.status(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_records_consistent(
    input_witness_reader: AccountCellDataReader,
    output_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        input_witness_reader.records(),
        output_witness_reader.records(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn distinguish_owner_and_manager(
    parser: &WitnessesParser,
    account_cell: usize,
    ref_cells: Vec<usize>,
    source: Source,
) -> Result<(Option<usize>, Option<usize>), Error> {
    debug!("Distinguish RefCells to OwnerCell and ManagerCell by AccountCell.witness, and panic if found unrelated RefCells.");

    if ref_cells.len() <= 0 {
        debug!(
            "Found AccountCell({})'s RefCells is empty in ({:?}).",
            account_cell, source
        );
        return Err(Error::AccountCellRefCellIsRequired);
    }

    let (_, _, entity) = parser.verify_and_get(account_cell, source)?;
    let input_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
        .map_err(|_| Error::WitnessEntityDecodingError)?;
    let expected_owner_lock = input_account_witness.owner_lock().into();
    let expected_manager_lock = input_account_witness.manager_lock().into();

    let mut owner_cell = None;
    let mut manager_cell = None;
    for index in ref_cells {
        let lock_script = load_cell_lock(index, source).map_err(|e| Error::from(e))?;

        // If owner lock and manager lock is the same, then distinguish owner and manager by cell data.
        if util::is_entity_eq(&expected_owner_lock, &expected_manager_lock) {
            let data = util::load_cell_data(index, source)?;
            if data_parser::ref_cell::get_is_owner(data) {
                owner_cell = Some(index);
            } else {
                manager_cell = Some(index);
            }
        } else {
            if util::is_entity_eq(&lock_script, &expected_owner_lock) {
                if owner_cell.is_some() {
                    debug!(
                        "Found AccountCell({})'s OwnerCell({}) is redundant in ({:?}) .",
                        account_cell, index, source
                    );
                    return Err(Error::AccountCellRedundantRefCellNotAllowed);
                }

                owner_cell = Some(index);
            } else if util::is_entity_eq(&lock_script, &expected_manager_lock) {
                if manager_cell.is_some() {
                    debug!(
                        "Found AccountCell({})'s ManagerCell({}) is redundant in ({:?}) .",
                        account_cell, index, source
                    );
                    return Err(Error::AccountCellRedundantRefCellNotAllowed);
                }

                manager_cell = Some(index);
            } else {
                debug!(
                    "Found AccountCell({}) and RefCell({}) is unrelated in source({:?}) .",
                    account_cell, index, source
                );
                return Err(Error::AccountCellUnrelatedRefCellFound);
            }
        }
    }

    Ok((owner_cell, manager_cell))
}
