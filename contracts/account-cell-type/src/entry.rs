use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    debug, high_level,
    high_level::{
        load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash, load_script,
    },
};
use core::convert::TryInto;
use das_core::{
    constants::{
        oracle_lock, super_lock, ScriptType, TypeScript, ALWAYS_SUCCESS_LOCK, DAS_WALLET_ID,
    },
    data_parser::account_cell,
    error::Error,
    util,
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
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        if old_cells.len() != 0 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }
        if new_cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        debug!("Check if super lock has been used in inputs ...");

        let super_lock = super_lock();
        let has_super_lock =
            util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len() > 0;
        if !has_super_lock {
            return Err(Error::SuperLockIsRequired);
        }

        debug!("Check if root AccountCell uses always_success lock ...");

        let index = new_cells[0];
        let always_success_script = util::script_literal_to_script(ALWAYS_SUCCESS_LOCK);
        let always_success_script_hash = util::blake2b_256(always_success_script.as_slice());
        let lock_script = load_cell_lock_hash(index, Source::Output).map_err(|e| Error::from(e))?;
        if lock_script != always_success_script_hash {
            return Err(Error::WalletRequireAlwaysSuccess);
        }
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

        let old_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Input,
        )?;
        let new_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Output,
        )?;
        let (old_account_cells, new_account_cells) = load_account_cells()?;

        verify_account_expiration(old_account_cells[0], timestamp)?;
        verify_account_consistent(old_account_cells[0], new_account_cells[0])?;

        debug!("Check the relationship between RefCells and AccountCell is correct.");

        // This will ensure that RefCells in inputs and outputs is unique and referenced by AccountCell.
        let (old_owner_cell, old_manager_cell) = distinguish_owner_and_manager(
            &parser,
            old_account_cells[0],
            old_ref_cells,
            Source::Input,
        )?;
        let (new_owner_cell, new_manager_cell) = distinguish_owner_and_manager(
            &parser,
            new_account_cells[0],
            new_ref_cells,
            Source::Output,
        )?;
        // Check if OwnerCell exists in inputs and outputs.
        if old_owner_cell.is_none() || new_owner_cell.is_none() {
            return Err(Error::AccountCellOwnerCellIsRequired);
        }
        // Check if ManagerCell not exists in inputs and outputs.
        if old_manager_cell.is_some() || new_manager_cell.is_some() {
            return Err(Error::AccountCellRedundantRefCellNotAllowed);
        }

        util::is_cell_only_lock_changed(
            (old_owner_cell.unwrap(), Source::Input),
            (new_owner_cell.unwrap(), Source::Output),
        )?;

        debug!(
            "Check if every fields except owner_lock and manager_lock in witness are consistent."
        );

        let (_, _, entity) = parser.verify_and_get(old_account_cells[0], Source::Input)?;
        let old_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let old_witness_reader = old_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(new_account_cells[0], Source::Output)?;
        let new_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let new_witness_reader = new_account_witness.as_reader();

        verify_if_id_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_account_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_registered_at_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_status_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_records_consistent(old_witness_reader, new_witness_reader)?;
    } else if action == b"edit_manager" {
        debug!("Route to transfer_account action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let timestamp = util::load_timestamp()?;

        let config = parser.configs().main()?;

        let old_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Input,
        )?;
        let new_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Output,
        )?;
        let (old_account_cells, new_account_cells) = load_account_cells()?;

        verify_account_expiration(old_account_cells[0], timestamp)?;
        verify_account_consistent(old_account_cells[0], new_account_cells[0])?;

        debug!("Check the relationship between RefCells and AccountCell is correct.");

        // This will ensure that RefCells in inputs and outputs is unique and referenced by AccountCell.
        let (old_owner_cell, old_manager_cell) = distinguish_owner_and_manager(
            &parser,
            old_account_cells[0],
            old_ref_cells,
            Source::Input,
        )?;
        let (new_owner_cell, new_manager_cell) = distinguish_owner_and_manager(
            &parser,
            new_account_cells[0],
            new_ref_cells,
            Source::Output,
        )?;
        // Check if OwnerCell exists in inputs and outputs.
        if old_owner_cell.is_none() || new_owner_cell.is_none() {
            return Err(Error::AccountCellOwnerCellIsRequired);
        }
        // Check if ManagerCell not exists in inputs and outputs.
        if old_manager_cell.is_none() || new_manager_cell.is_none() {
            return Err(Error::AccountCellManagerCellIsRequired);
        }

        util::is_cell_only_lock_changed(
            (old_manager_cell.unwrap(), Source::Input),
            (new_manager_cell.unwrap(), Source::Output),
        )?;

        debug!("Check if every fields except manager_lock in witness are consistent.");

        let (_, _, entity) = parser.verify_and_get(old_account_cells[0], Source::Input)?;
        let old_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let old_witness_reader = old_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(new_account_cells[0], Source::Output)?;
        let new_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let new_witness_reader = new_account_witness.as_reader();

        verify_if_id_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_account_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_owner_lock_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_registered_at_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_status_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_records_consistent(old_witness_reader, new_witness_reader)?;
    } else if action == b"edit_records" {
        debug!("Route to transfer_account action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;

        let config = parser.configs().main()?;
        let timestamp = util::load_timestamp()?;

        let old_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Input,
        )?;
        let new_ref_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().ref_cell(),
            Source::Output,
        )?;
        let (old_account_cells, new_account_cells) = load_account_cells()?;

        verify_account_expiration(old_account_cells[0], timestamp)?;
        verify_account_consistent(old_account_cells[0], new_account_cells[0])?;

        debug!("Check the relationship between RefCells and AccountCell is correct.");

        // This will ensure that RefCells in inputs and outputs is unique and referenced by AccountCell.
        let (old_owner_cell, old_manager_cell) = distinguish_owner_and_manager(
            &parser,
            old_account_cells[0],
            old_ref_cells,
            Source::Input,
        )?;
        let (new_owner_cell, new_manager_cell) = distinguish_owner_and_manager(
            &parser,
            new_account_cells[0],
            new_ref_cells,
            Source::Output,
        )?;
        // Check if OwnerCell exists in inputs and outputs.
        if old_owner_cell.is_some() || new_owner_cell.is_some() {
            return Err(Error::AccountCellRedundantRefCellNotAllowed);
        }
        // Check if ManagerCell not exists in inputs and outputs.
        if old_manager_cell.is_none() || new_manager_cell.is_none() {
            return Err(Error::AccountCellManagerCellIsRequired);
        }

        debug!("Check if every fields except records in witness are consistent.");

        let (_, _, entity) = parser.verify_and_get(old_account_cells[0], Source::Input)?;
        let old_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let old_witness_reader = old_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(new_account_cells[0], Source::Output)?;
        let new_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let new_witness_reader = new_account_witness.as_reader();

        verify_if_id_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_account_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_owner_lock_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_manager_lock_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_registered_at_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_status_consistent(old_witness_reader, new_witness_reader)?;
    } else if action == b"renew_account" {
        debug!("Route to transfer_account action ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[ConfigID::ConfigCellMain, ConfigID::ConfigCellRegister])?;

        let config_main = parser.configs().main()?;
        let config_register = parser.configs().register()?;

        let (old_account_cells, new_account_cells) = load_account_cells()?;

        verify_account_consistent(old_account_cells[0], new_account_cells[0])?;

        debug!("Check if every fields except registered_at in witness are consistent.");

        let (_, _, entity) = parser.verify_and_get(old_account_cells[0], Source::Input)?;
        let old_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let old_witness_reader = old_account_witness.as_reader();
        let (_, _, entity) = parser.verify_and_get(new_account_cells[0], Source::Output)?;
        let new_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let new_witness_reader = new_account_witness.as_reader();

        verify_if_id_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_account_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_owner_lock_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_manager_lock_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_status_consistent(old_witness_reader, new_witness_reader)?;
        verify_if_records_consistent(old_witness_reader, new_witness_reader)?;

        debug!("Check if the renewal duration is longer than or equal to one year.");

        let old_registered_at = u64::from(old_witness_reader.registered_at());
        let new_registered_at = u64::from(new_witness_reader.registered_at());
        let duration = new_registered_at - old_registered_at;

        if duration < 86400 * 365 {
            return Err(Error::AccountCellRenewDurationMustLongerThanYear);
        }

        debug!("Check if the registered_at field has been updated correctly based on the capacity paid by the user.");

        let length_in_price = util::get_length_in_price(old_witness_reader.account().len() as u64);
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
        let (old_wallet_cells, new_wallet_cells) = load_wallet_cells(config_main)?;

        if old_wallet_cells.len() != 1 || new_wallet_cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        // Get the current quotation of CKB/USD from QuoteCell.
        let quote_lock = oracle_lock();
        let quote_cells =
            util::find_cells_by_script(ScriptType::Lock, &quote_lock, Source::CellDep)?;

        if quote_cells.len() != 1 {
            return Err(Error::QuoteCellIsRequired);
        }

        let quote_cell_data =
            load_cell_data(quote_cells[0], Source::CellDep).map_err(|e| Error::from(e))?;
        let quote = u64::from_le_bytes(quote_cell_data.try_into().unwrap()); // y CKB/USD

        let old_wallet_capacity =
            load_cell_capacity(old_wallet_cells[0], Source::Input).map_err(|e| Error::from(e))?;
        let new_wallet_capacity =
            load_cell_capacity(new_wallet_cells[0], Source::Output).map_err(|e| Error::from(e))?;

        // Renew price for 1 year in CKB = x ÷ y .
        let renew_price = renew_price_in_usd / quote * 100_000_000;

        let expected_duration =
            (new_wallet_capacity - old_wallet_capacity) * 86400 * 365 / renew_price;
        if duration > expected_duration {
            debug!("Verify is user payed enough capacity: {}[duration] > ({}[after_ckb] - {}[before_ckb]) * 86400 * 365 / {}[renew_price] -> true",
                   duration,
                   new_wallet_capacity,
                   old_wallet_capacity,
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
        let (old_account_cells, new_account_cells) = load_account_cells()?;
        if old_account_cells.len() != 1 || new_account_cells.len() != 0 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        debug!("Check if account has reached the end off the expiration grace period.");

        let expiration_grace_period =
            u32::from(config_main.account_expiration_grace_period()) as u64;
        let account_data = util::load_cell_data(old_account_cells[0], Source::Input)?;
        let expired_at = account_cell::get_expired_at(&account_data);
        if expired_at + expiration_grace_period >= timestamp {
            return Err(Error::AccountCellIsNotExpired);
        }

        let account_id = account_cell::get_id(&account_data);

        debug!("Check if the transaction has required WalletCells.");

        let (old_wallet_cells, new_wallet_cells) = load_wallet_cells(config_main)?;

        // There should be a WalletCell of the account and a WalletCell of DAS in inputs.
        let mut account_wallet = None;
        let mut old_das_wallet = None;
        for index in old_wallet_cells {
            let type_script = high_level::load_cell_type(index, Source::Input)
                .map_err(|e| Error::from(e))?
                .unwrap();
            let id = type_script.as_reader().args().raw_data();
            if id == account_id {
                account_wallet = Some(index);
            } else if id == &DAS_WALLET_ID {
                old_das_wallet = Some(index);
            } else {
                return Err(Error::AccountCellFoundInvalidTransaction);
            }
        }

        // The WalletCell of the account should be recycled either.
        if new_wallet_cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        let type_script = high_level::load_cell_type(new_wallet_cells[0], Source::Input)
            .map_err(|e| Error::from(e))?
            .unwrap();
        let id = type_script.as_reader().args().raw_data();
        if id == &DAS_WALLET_ID {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }
        let new_das_wallet = new_wallet_cells[0];

        debug!("Check if the DAS WalletCell's balance has increased correctly.");

        let account_wallet_occupied_capacity =
            high_level::load_cell_occupied_capacity(account_wallet.unwrap(), Source::Input)
                .map_err(|e| Error::from(e))?;
        let old_capacity = high_level::load_cell_capacity(old_das_wallet.unwrap(), Source::Input)
            .map_err(|e| Error::from(e))?;
        let new_capacity = high_level::load_cell_capacity(new_das_wallet, Source::Output)
            .map_err(|e| Error::from(e))?;
        if new_capacity - old_capacity < account_wallet_occupied_capacity {
            debug!(
                "Compare recycle capacity: {}[new_capacity] - {}[old_capacity] < {}[account_wallet_occupied_capacity] => true",
                new_capacity,
                old_capacity,
                account_wallet_occupied_capacity
            );
            return Err(Error::AccountCellRecycleCapacityError);
        }

        debug!("Check if the User's owner lock get correct change.");

        let (_, _, entity) = parser.verify_and_get(old_account_cells[0], Source::Input)?;
        let owner_lock = AccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?
            .owner_lock();
        let cells =
            util::find_cells_by_script(ScriptType::Lock, &owner_lock.into(), Source::Output)?;

        if cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        let account_cell_capacity =
            high_level::load_cell_capacity(old_account_cells[0], Source::Input)
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
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        debug!("Check if AccountCell is consistent.");

        if old_cells.len() != new_cells.len() {
            return Err(Error::CellsMustHaveSameOrderAndNumber);
        }

        for (i, old_index) in old_cells.into_iter().enumerate() {
            let new_index = new_cells[i];
            util::is_cell_capacity_equal((old_index, Source::Input), (new_index, Source::Output))?;
            util::is_cell_consistent((old_index, Source::Input), (new_index, Source::Output))?;
        }
    }

    Ok(())
}

fn load_account_cells() -> Result<(Vec<usize>, Vec<usize>), Error> {
    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let old_account_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let new_account_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

    Ok((old_account_cells, new_account_cells))
}

fn load_wallet_cells(config: ConfigCellMainReader) -> Result<(Vec<usize>, Vec<usize>), Error> {
    let old_wallet_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        config.type_id_table().wallet_cell(),
        Source::Input,
    )?;
    let new_wallet_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        config.type_id_table().wallet_cell(),
        Source::Output,
    )?;

    Ok((old_wallet_cells, new_wallet_cells))
}

fn verify_account_consistent(
    old_account_index: usize,
    new_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if everything consistent except data in the AccountCell.");

    util::is_cell_capacity_equal(
        (old_account_index, Source::Input),
        (new_account_index, Source::Output),
    )?;
    util::is_cell_lock_equal(
        (old_account_index, Source::Input),
        (new_account_index, Source::Output),
    )?;
    util::is_cell_type_equal(
        (old_account_index, Source::Input),
        (new_account_index, Source::Output),
    )?;

    debug!("Check if the data of AccountCell only changed leading 32 bytes.");

    let old_data = load_cell_data(old_account_index, Source::Input).map_err(|e| Error::from(e))?;
    let new_data = load_cell_data(new_account_index, Source::Output).map_err(|e| Error::from(e))?;

    if old_data.get(32..).unwrap() != new_data.get(32..).unwrap() {
        return Err(Error::AccountCellDataNotConsistent);
    }

    Ok(())
}

fn verify_account_expiration(account_cell_index: usize, current: u64) -> Result<(), Error> {
    debug!("Check if AccountCell is expired.");

    let data = load_cell_data(account_cell_index, Source::Input).map_err(|e| Error::from(e))?;
    let expired_at = account_cell::get_expired_at(data.as_slice());

    if current > expired_at {
        if current - expired_at > 86400 * 30 {
            return Err(Error::AccountCellHasExpired);
        } else {
            return Err(Error::AccountCellInExpirationGracePeriod);
        }
    }

    Ok(())
}

fn verify_if_id_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(old_witness_reader.id(), new_witness_reader.id()) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_owner_lock_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        old_witness_reader.owner_lock(),
        new_witness_reader.owner_lock(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_manager_lock_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        old_witness_reader.manager_lock(),
        new_witness_reader.manager_lock(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_account_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(old_witness_reader.account(), new_witness_reader.account()) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_registered_at_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(
        old_witness_reader.registered_at(),
        new_witness_reader.registered_at(),
    ) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_status_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(old_witness_reader.status(), new_witness_reader.status()) {
        return Err(Error::AccountCellProtectFieldIsModified);
    }

    Ok(())
}

fn verify_if_records_consistent(
    old_witness_reader: AccountCellDataReader,
    new_witness_reader: AccountCellDataReader,
) -> Result<(), Error> {
    if !util::is_reader_eq(old_witness_reader.records(), new_witness_reader.records()) {
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
    let old_account_witness = AccountCellData::from_slice(entity.as_reader().raw_data())
        .map_err(|_| Error::WitnessEntityDecodingError)?;
    let expected_owner_lock = old_account_witness.owner_lock().into();
    let expected_manager_lock = old_account_witness.manager_lock().into();

    let mut owner_cell = None;
    let mut manager_cell = None;
    for index in ref_cells {
        let lock_script = load_cell_lock(index, source).map_err(|e| Error::from(e))?;

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

    Ok((owner_cell, manager_cell))
}
