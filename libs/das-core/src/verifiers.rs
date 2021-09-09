use alloc::prelude::v1::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::{debug, high_level};
use das_types::constants::LockRole;
use das_types::packed::ConfigCellAccountReader;

use crate::{assert, data_parser, error::Error, util, warn};
use alloc::boxed::Box;
use das_types::mixer::AccountCellDataReaderMixer;

pub fn verify_unlock_role(params: &[u8], lock: LockRole) -> Result<(), Error> {
    debug!("Check if transaction is unlocked by {:?}.", lock);

    assert!(
        params.len() > 0 && params[0] == lock as u8,
        Error::AccountCellPermissionDenied,
        "This transaction should be unlocked by the {:?}'s signature.",
        lock
    );

    Ok(())
}

pub fn verify_account_expiration(
    config: ConfigCellAccountReader,
    account_cell_index: usize,
    current: u64,
) -> Result<(), Error> {
    debug!("Check if AccountCell is expired.");

    let data = util::load_cell_data(account_cell_index, Source::Input)?;
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

pub fn verify_account_lock_consistent(
    input_account_index: usize,
    output_account_index: usize,
    changed_lock: Option<&str>,
) -> Result<(), Error> {
    debug!("Check if lock consistent in the AccountCell.");

    if let Some(lock) = changed_lock {
        let input_lock = high_level::load_cell_lock(input_account_index, Source::Input).map_err(|e| Error::from(e))?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock =
            high_level::load_cell_lock(output_account_index, Source::Output).map_err(|e| Error::from(e))?;
        let output_args = output_lock.as_reader().args().raw_data();

        if lock == "owner" {
            assert!(
                data_parser::das_lock_args::get_owner_lock_args(input_args)
                    != data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellOwnerLockShouldBeModified,
                "The owner lock args in AccountCell.lock should be different in input and output."
            );

            assert!(
                data_parser::das_lock_args::get_manager_lock_args(output_args)
                    == data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellManagerLockShouldBeModified,
                "The manager lock args in AccountCell.lock should be different in input and output."
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

pub fn verify_account_data_consistent(
    input_account_index: usize,
    output_account_index: usize,
    except: Vec<&str>,
) -> Result<(), Error> {
    debug!("Check if AccountCell.data is consistent in input and output.");

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let output_data = util::load_cell_data(output_account_index, Source::Output)?;

    assert!(
        data_parser::account_cell::get_id(&input_data) == data_parser::account_cell::get_id(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.id field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        data_parser::account_cell::get_next(&input_data) == data_parser::account_cell::get_next(&output_data),
        Error::AccountCellDataNotConsistent,
        "The data.next field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    assert!(
        data_parser::account_cell::get_account(&input_data) == data_parser::account_cell::get_account(&output_data),
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

pub fn verify_account_capacity_not_decrease(
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Error> {
    debug!("Check if capacity consistent in the AccountCell.");

    let input = high_level::load_cell_capacity(input_account_index, Source::Input).map_err(|e| Error::from(e))?;
    let output = high_level::load_cell_capacity(output_account_index, Source::Output).map_err(|e| Error::from(e))?;

    // ⚠️ Equal is not allowed here because we want to avoid abuse cell.
    assert!(
        input <= output,
        Error::CellLockCanNotBeModified,
        "The capacity of the AccountCell should be consistent or increased.(input: {}, output: {})",
        input,
        output
    );

    Ok(())
}

pub fn verify_account_witness_consistent<'a>(
    input_index: usize,
    output_index: usize,
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    except: Vec<&str>,
) -> Result<(), Error> {
    debug!("Check if AccountCell.witness is consistent in input and output.");

    macro_rules! assert_field_consistent {
        ($input_witness_reader:expr, $output_witness_reader:expr, $( ($field:ident, $field_name:expr) ),*) => {
            $(
                assert!(
                    util::is_reader_eq(
                        $input_witness_reader.$field(),
                        $output_witness_reader.$field()
                    ),
                    Error::AccountCellProtectFieldIsModified,
                    "The witness.{} field of inputs[{}] and outputs[{}] should be the same.",
                    $field_name,
                    input_index,
                    output_index
                );
            )*
        };
    }

    macro_rules! assert_field_consistent_if_not_except {
        ($input_witness_reader:expr, $output_witness_reader:expr, $( ($field:ident, $field_name:expr) ),*) => {
            $(
                if !except.contains(&$field_name) {
                    assert_field_consistent!(
                        $input_witness_reader,
                        $output_witness_reader,
                        ($field, $field_name)
                    );
                }
            )*
        };
    }

    let output_witness_reader = output_witness_reader
        .try_into_latest()
        .map_err(|_| Error::NarrowMixerTypeFailed)?;
    // Migration for AccountCellData v1
    if input_witness_reader.version() == 1 {
        let input_witness_reader = input_witness_reader
            .try_into_v1()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

        assert_field_consistent!(
            input_witness_reader,
            output_witness_reader,
            (id, "id"),
            (account, "account"),
            (registered_at, "registered_at"),
            (status, "status")
        );

        assert_field_consistent_if_not_except!(input_witness_reader, output_witness_reader, (records, "records"));
    } else {
        let input_witness_reader = input_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

        assert_field_consistent!(
            input_witness_reader,
            output_witness_reader,
            (id, "id"),
            (account, "account"),
            (registered_at, "registered_at"),
            (status, "status")
        );

        assert_field_consistent_if_not_except!(
            input_witness_reader,
            output_witness_reader,
            (records, "records"),
            (last_transfer_account_at, "last_transfer_account_at"),
            (last_edit_manager_at, "last_edit_manager_at"),
            (last_edit_records_at, "last_edit_records_at")
        );
    }

    Ok(())
}
