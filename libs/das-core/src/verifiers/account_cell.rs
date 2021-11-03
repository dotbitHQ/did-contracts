use crate::{
    assert, constants::DasLockType, constants::*, data_parser, error::Error, util, warn,
    witness_parser::WitnessesParser,
};
use alloc::{boxed::Box, string::String, vec::Vec};
use ckb_std::{ckb_constants::Source, debug, high_level};
use das_types::{constants::*, mixer::AccountCellDataReaderMixer, packed::*, util as das_types_util};

pub fn verify_unlock_role(action: BytesReader, params: &[BytesReader]) -> Result<(), Error> {
    let required_role_opt = util::get_action_required_role(action);
    if required_role_opt.is_none() {
        debug!("Skip checking the required role of the transaction.");
        return Ok(());
    }

    debug!("Check if the transaction is unlocked by expected role.");

    assert!(
        params.len() > 0,
        Error::AccountCellPermissionDenied,
        "This transaction should have a role param."
    );

    let required_role = required_role_opt.unwrap();
    // It is a convention that the param of role should always be the last param.
    let current_role = params[params.len() - 1].raw_data()[0];

    assert!(
        current_role == required_role as u8,
        Error::AccountCellPermissionDenied,
        "This transaction should be unlocked by the {:?}'s signature.",
        required_role
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
    debug!(
        "Check if lock consistent in the AccountCell.(changed_lock: {:?})",
        changed_lock
    );

    if let Some(lock) = changed_lock {
        let input_lock = high_level::load_cell_lock(input_account_index, Source::Input).map_err(Error::from)?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock = high_level::load_cell_lock(output_account_index, Source::Output).map_err(Error::from)?;
        let output_args = output_lock.as_reader().args().raw_data();

        if lock == "owner" {
            assert!(
                data_parser::das_lock_args::get_owner_lock_args(input_args)
                    != data_parser::das_lock_args::get_owner_lock_args(output_args),
                Error::AccountCellOwnerLockShouldBeModified,
                "The owner lock args in AccountCell.lock should be different in inputs and outputs."
            );

            // When owner is changed, there is no need to verify the manager's consistent any more.
        } else {
            let input_lock_type = data_parser::das_lock_args::get_owner_type(input_args);
            let input_pubkey_hash = data_parser::das_lock_args::get_owner_lock_args(input_args);
            let output_lock_type = data_parser::das_lock_args::get_owner_type(output_args);
            let output_pubkey_hash = data_parser::das_lock_args::get_owner_lock_args(output_args);

            let lock_type_consistent = if input_lock_type == DasLockType::ETH as u8 {
                output_lock_type == input_lock_type || output_lock_type == DasLockType::ETHTypedData as u8
            } else {
                output_lock_type == input_lock_type
            };
            assert!(
                lock_type_consistent && input_pubkey_hash == output_pubkey_hash,
                Error::AccountCellOwnerLockShouldNotBeModified,
                "The owner lock args in AccountCell.lock should be consistent in inputs and outputs."
            );

            assert!(
                data_parser::das_lock_args::get_manager_lock_args(input_args)
                    != data_parser::das_lock_args::get_manager_lock_args(output_args),
                Error::AccountCellManagerLockShouldBeModified,
                "The manager lock args in AccountCell.lock should be different in inputs and outputs."
            );
        }
    } else {
        let input_lock = high_level::load_cell_lock(input_account_index, Source::Input).map_err(Error::from)?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock = high_level::load_cell_lock(output_account_index, Source::Output).map_err(Error::from)?;
        let output_args = output_lock.as_reader().args().raw_data();

        assert!(
            util::is_type_id_equal(input_lock.as_reader(), output_lock.as_reader()),
            Error::CellLockCanNotBeModified,
            "The AccountCell.lock should be consistent in inputs and outputs."
        );

        macro_rules! verify_lock_consistent {
            ($role:expr, $fn_get_type:ident, $fn_get_args:ident) => {{
                let input_lock_type = data_parser::das_lock_args::$fn_get_type(input_args);
                let input_pubkey_hash = data_parser::das_lock_args::$fn_get_args(input_args);
                let output_lock_type = data_parser::das_lock_args::$fn_get_type(output_args);
                let output_pubkey_hash = data_parser::das_lock_args::$fn_get_args(output_args);

                let lock_type_consistent = if input_lock_type == DasLockType::ETH as u8 {
                    output_lock_type == input_lock_type || output_lock_type == DasLockType::ETHTypedData as u8
                } else {
                    output_lock_type == input_lock_type
                };
                assert!(
                    lock_type_consistent && input_pubkey_hash == output_pubkey_hash,
                    Error::CellLockCanNotBeModified,
                    "The pubkey hash of AccountCell's owner should be consistent in inputs and outputs."
                );
            }};
        }

        verify_lock_consistent!("owner", get_owner_type, get_owner_lock_args);
        verify_lock_consistent!("manager", get_manager_type, get_manager_lock_args);
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

    let input = high_level::load_cell_capacity(input_account_index, Source::Input).map_err(Error::from)?;
    let output = high_level::load_cell_capacity(output_account_index, Source::Output).map_err(Error::from)?;

    // ⚠️ Equal is not allowed here because we want to avoid abuse cell.
    assert!(
        input <= output,
        Error::AccountCellChangeCapacityError,
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
            (registered_at, "registered_at")
        );

        assert_field_consistent_if_not_except!(
            input_witness_reader,
            output_witness_reader,
            (records, "records"),
            (last_transfer_account_at, "last_transfer_account_at"),
            (last_edit_manager_at, "last_edit_manager_at"),
            (last_edit_records_at, "last_edit_records_at"),
            (status, "status")
        );
    }

    Ok(())
}

pub fn verify_account_cell_status_update_correctly<'a>(
    input_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    expected_input_status: AccountStatus,
    expected_output_status: AccountStatus,
) -> Result<(), Error> {
    if input_account_cell_witness_reader.version() == 1 {
        // There is no version 1 AccountCell in mainnet, so we simply disable them here.
        unreachable!()
    } else {
        let input_account_cell_witness_reader = input_account_cell_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;
        let output_account_cell_witness_reader = output_account_cell_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

        let input_status = u8::from(input_account_cell_witness_reader.status());
        let output_status = u8::from(output_account_cell_witness_reader.status());

        assert!(
            input_status == expected_input_status as u8,
            Error::AccountCellStatusLocked,
            "The AccountCell.witness.status should be {:?} in inputs.",
            expected_input_status
        );
        assert!(
            output_status == expected_output_status as u8,
            Error::AccountCellStatusLocked,
            "The AccountCell.witness.status should be {:?} in outputs.",
            expected_output_status
        );
    }

    Ok(())
}

pub fn verify_account_cell_status<'a>(
    account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    expected_status: AccountStatus,
    index: usize,
    source: Source,
) -> Result<(), Error> {
    if account_cell_witness_reader.version() == 1 {
        // There is no version 1 AccountCell in mainnet, so we simply disable them here.
        unreachable!()
    } else {
        let account_cell_witness_reader = account_cell_witness_reader
            .try_into_latest()
            .map_err(|_| Error::NarrowMixerTypeFailed)?;

        let account_cell_status = u8::from(account_cell_witness_reader.status());

        assert!(
            account_cell_status == expected_status as u8,
            Error::AccountCellStatusLocked,
            "{:?}[{}]The AccountCell.witness.status should be {:?}.",
            source,
            index,
            expected_status
        );
    }

    Ok(())
}

pub fn verify_preserved_accounts(parser: &mut WitnessesParser, account: &[u8]) -> Result<(), Error> {
    debug!("Verify if account is preserved.");

    let account_hash = util::blake2b_256(account);
    let account_id = account_hash.get(..ACCOUNT_ID_LENGTH).unwrap();
    let index = (account_id[0] % PRESERVED_ACCOUNT_CELL_COUNT) as usize;
    let data_type = das_types_util::preserved_accounts_group_to_data_type(index);

    // debug!(
    //     "account: {}, account ID: {:?}",
    //     String::from_utf8(account.to_vec()).unwrap(),
    //     account_id
    // );

    parser.parse_config(&[data_type])?;
    let preserved_accounts = parser.configs.preserved_account()?;

    if is_account_id_in_collection(account_id, preserved_accounts) {
        warn!(
            "Account {} is preserved. (hex: 0x{}, hash: 0x{})",
            String::from_utf8(account.to_vec()).unwrap(),
            util::hex_string(account),
            util::hex_string(&account_hash)
        );
        return Err(Error::AccountIsPreserved);
    }

    Ok(())
}

/**
check if the account is an account that can never be registered.
 **/
pub fn verify_unavailable_accounts(parser: &mut WitnessesParser, account: &[u8]) -> Result<(), Error> {
    debug!("Verify if account if unavailable");

    parser.parse_config(&[DataType::ConfigCellUnAvailableAccount])?;

    let account_hash = util::blake2b_256(account);
    let account_id = account_hash.get(..ACCOUNT_ID_LENGTH).unwrap();
    let unavailable_accounts = parser.configs.unavailable_account()?;

    if is_account_id_in_collection(account_id, unavailable_accounts) {
        warn!(
            "Account {} is unavailable. (hex: 0x{}, hash: 0x{})",
            String::from_utf8(account.to_vec()).unwrap(),
            util::hex_string(account),
            util::hex_string(&account_hash)
        );
        return Err(Error::AccountIsUnAvailable);
    }

    Ok(())
}

fn is_account_id_in_collection(account_id: &[u8], collection: &[u8]) -> bool {
    let length = collection.len();

    let first = &collection[0..20];
    let last = &collection[length - 20..];

    return if account_id < first {
        debug!("The account is less than the first preserved account, skip.");
        false
    } else if account_id > last {
        debug!("The account is bigger than the last preserved account, skip.");
        false
    } else {
        let accounts_total = collection.len() / ACCOUNT_ID_LENGTH;
        let mut start_account_index = 0;
        let mut end_account_index = accounts_total - 1;

        loop {
            let mid_account_index = (start_account_index + end_account_index) / 2;
            // debug!("mid_account_index = {:?}", mid_account_index);
            let mid_account_start_byte_index = mid_account_index * ACCOUNT_ID_LENGTH;
            let mid_account_end_byte_index = mid_account_start_byte_index + ACCOUNT_ID_LENGTH;
            let mid_account_bytes = collection
                .get(mid_account_start_byte_index..mid_account_end_byte_index)
                .unwrap();

            if mid_account_bytes < account_id {
                start_account_index = mid_account_index + 1;
                // debug!("<");
            } else if mid_account_bytes > account_id {
                // debug!(">");
                end_account_index = if mid_account_index > 1 {
                    mid_account_index - 1
                } else {
                    0
                };
            } else {
                return true;
            }

            if start_account_index > end_account_index || end_account_index == 0 {
                break;
            }
        }

        false
    };
}
