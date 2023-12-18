use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_types::constants::{das_lock, *};
use das_types::mixer::AccountCellDataReaderMixer;
use das_types::packed::*;
use das_types::util as types_util;

use crate::config::Config;
use crate::constants::*;
use crate::error::*;
#[cfg(debug_assertions)]
use crate::util::print_dp;
use crate::util::{blake2b_256, find_cells_by_script};
use crate::{data_parser, util};

pub fn verify_unlock_role(action: Action, role: Option<LockRole>) -> Result<(), Box<dyn ScriptError>> {
    let required_role_opt = types_util::get_action_required_sign_role(action);
    if required_role_opt.is_none() {
        debug!("Skip checking the required role of the transaction.");
        return Ok(());
    }

    debug!("Check if the transaction is unlocked by expected role.");

    das_assert!(
        required_role_opt == role,
        AccountCellErrorCode::AccountCellPermissionDenied,
        "This transaction should be unlocked by the {:?}'s signature.",
        required_role_opt.unwrap()
    );

    Ok(())
}

pub fn verify_account_expiration(
    config: ConfigCellAccountReader,
    index: usize,
    source: Source,
    current_timestamp: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("{:?}[{}] Verify if the AccountCell is expired.", source, index);

    let data = util::load_cell_data(index, source)?;
    let expired_at = data_parser::account_cell::get_expired_at(data.as_slice());
    let expiration_grace_period = u32::from(config.expiration_grace_period()) as u64;
    let expiration_auction_period = u32::from(config.expiration_auction_period()) as u64;
    let expiration_deliver_period = u32::from(config.expiration_deliver_period()) as u64;

    if current_timestamp > expired_at {
        let duration = current_timestamp - expired_at;
        if duration > expiration_grace_period + expiration_auction_period + expiration_deliver_period {
            warn!("The AccountCell has been expired. Will be recycled soon.");
            return Err(code_to_error!(AccountCellErrorCode::AccountCellHasExpired));
        } else if duration > expiration_grace_period + expiration_auction_period {
            warn!("The AccountCell has been in expiration auction confirmation period.");
            return Err(code_to_error!(
                AccountCellErrorCode::AccountCellInExpirationAuctionConfirmationPeriod
            ));
        } else if duration > expiration_grace_period {
            warn!("The AccountCell has been in expiration auction period.");
            return Err(code_to_error!(
                AccountCellErrorCode::AccountCellInExpirationAuctionPeriod
            ));
        } else {
            warn!("The AccountCell has been in expiration grace period. Need to be renew as soon as possible.");
            return Err(code_to_error!(AccountCellErrorCode::AccountCellInExpirationGracePeriod));
        }
    }

    Ok(())
}

pub fn verify_account_in_auction(
    config: ConfigCellAccountReader,
    index: usize,
    source: Source,
    current_timestamp: u64,
    bid_price: u64,
    basic_price: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "{:?}[{}] Verify whether this account is in the Dutch auction period.",
        source, index
    );

    let data = util::load_cell_data(index, source)?;
    let expired_at = data_parser::account_cell::get_expired_at(data.as_slice());
    let expiration_grace_period = u32::from(config.expiration_grace_period()) as u64;
    let expiration_auction_period = u32::from(config.expiration_auction_period()) as u64;
    let expiration_auction_start_premium = u32::from(config.expiration_auction_start_premiums()) as u64;

    if current_timestamp > expired_at {
        let duration_after_expired = current_timestamp - expired_at;
        let auction_start_time = expiration_grace_period;
        let auction_end_time = auction_start_time + expiration_auction_period;

        if duration_after_expired > auction_end_time {
            warn!("The expired account auction has ended. Will be recycled soon.");
            return Err(code_to_error!(AccountCellErrorCode::AccountCellHasExpired));
        } else if duration_after_expired < auction_start_time {
            warn!("The AccountCell has been in expiration grace period. Cannot conduct auction.");
            return Err(code_to_error!(AccountCellErrorCode::AccountCellInExpirationGracePeriod));
        }

        //Check whether the bidding price is less than the expected price
        let duration_in_auction = duration_after_expired - auction_start_time;
        let premium = util::calculate_dutch_auction_premium(duration_in_auction, expiration_auction_start_premium);

        let expected_price = basic_price + premium;

        debug!("The expected price is {} .", print_dp(&expected_price));

        if bid_price < expected_price {
            warn!(
                "The bid is too low and the auction fails. The expected price is {} the actual price is {}.",
                expected_price, bid_price
            );
            return Err(code_to_error!(AccountCellErrorCode::AccountCellBidPriceTooLow));
        }
    } else {
        warn!("The AccountCell has not been expired.");
        return Err(code_to_error!(AccountCellErrorCode::AccountCellIsNotExpired));
    }

    Ok(())
}
pub fn verify_account_lock_consistent(
    input_account_index: usize,
    output_account_index: usize,
    changed_lock: Option<&str>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "Check if lock consistent in the AccountCell.(changed_lock: {:?})",
        changed_lock
    );

    if let Some(lock) = changed_lock {
        let input_lock =
            high_level::load_cell_lock(input_account_index, Source::Input).map_err(Error::<ErrorCode>::from)?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock =
            high_level::load_cell_lock(output_account_index, Source::Output).map_err(Error::<ErrorCode>::from)?;
        let output_args = output_lock.as_reader().args().raw_data();

        if lock == "owner" {
            das_assert!(
                data_parser::das_lock_args::get_owner_type(input_args)
                    != data_parser::das_lock_args::get_owner_type(output_args)
                    || data_parser::das_lock_args::get_owner_lock_args(input_args)
                        != data_parser::das_lock_args::get_owner_lock_args(output_args),
                AccountCellErrorCode::AccountCellOwnerLockShouldBeModified,
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
            das_assert!(
                lock_type_consistent && input_pubkey_hash == output_pubkey_hash,
                AccountCellErrorCode::AccountCellOwnerLockShouldNotBeModified,
                "The owner lock args in AccountCell.lock should be consistent in inputs and outputs."
            );

            das_assert!(
                data_parser::das_lock_args::get_manager_type(input_args)
                    != data_parser::das_lock_args::get_manager_type(output_args)
                    || data_parser::das_lock_args::get_manager_lock_args(input_args)
                        != data_parser::das_lock_args::get_manager_lock_args(output_args),
                AccountCellErrorCode::AccountCellManagerLockShouldBeModified,
                "The manager lock args in AccountCell.lock should be different in inputs and outputs."
            );
        }
    } else {
        let input_lock =
            high_level::load_cell_lock(input_account_index, Source::Input).map_err(Error::<ErrorCode>::from)?;
        let input_args = input_lock.as_reader().args().raw_data();
        let output_lock =
            high_level::load_cell_lock(output_account_index, Source::Output).map_err(Error::<ErrorCode>::from)?;
        let output_args = output_lock.as_reader().args().raw_data();

        das_assert!(
            util::is_type_id_equal(input_lock.as_reader(), output_lock.as_reader()),
            ErrorCode::CellLockCanNotBeModified,
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
                das_assert!(
                    lock_type_consistent && input_pubkey_hash == output_pubkey_hash,
                    ErrorCode::CellLockCanNotBeModified,
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
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if AccountCell.data is consistent in input and output.");

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let output_data = util::load_cell_data(output_account_index, Source::Output)?;

    das_assert!(
        data_parser::account_cell::get_id(&input_data) == data_parser::account_cell::get_id(&output_data),
        AccountCellErrorCode::AccountCellDataNotConsistent,
        "The data.id field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    if !except.contains(&"next") {
        das_assert!(
            data_parser::account_cell::get_next(&input_data) == data_parser::account_cell::get_next(&output_data),
            AccountCellErrorCode::AccountCellDataNotConsistent,
            "The data.next field of inputs[{}] and outputs[{}] should be the same.",
            input_account_index,
            output_account_index
        );
    }
    das_assert!(
        data_parser::account_cell::get_account(&input_data) == data_parser::account_cell::get_account(&output_data),
        AccountCellErrorCode::AccountCellDataNotConsistent,
        "The data.account field of inputs[{}] and outputs[{}] should be the same.",
        input_account_index,
        output_account_index
    );
    if !except.contains(&"expired_at") {
        let input_expired_at = data_parser::account_cell::get_expired_at(&input_data);
        let output_expired_at = data_parser::account_cell::get_expired_at(&output_data);

        das_assert!(
            input_expired_at == output_expired_at,
            AccountCellErrorCode::AccountCellDataNotConsistent,
            "The data.expired_at field of inputs[{}] and outputs[{}] should be the same. (inputs: {}, outputs: {})",
            input_account_index,
            output_account_index,
            input_expired_at,
            output_expired_at
        );
    }

    Ok(())
}

pub fn verify_account_capacity_not_decrease(
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if capacity consistent in the AccountCell.");

    let input = high_level::load_cell_capacity(input_account_index, Source::Input).map_err(Error::<ErrorCode>::from)?;
    let output =
        high_level::load_cell_capacity(output_account_index, Source::Output).map_err(Error::<ErrorCode>::from)?;

    das_assert!(
        input <= output,
        AccountCellErrorCode::AccountCellChangeCapacityError,
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
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if AccountCell.witness is consistent in input and output.");

    macro_rules! das_assert_field_consistent {
        ($input_witness_reader:expr, $output_witness_reader:expr, $( ($field:ident, $field_name:expr) ),*) => {
            $(
                das_assert!(
                    util::is_reader_eq(
                        $input_witness_reader.$field(),
                        $output_witness_reader.$field()
                    ),
                    AccountCellErrorCode::AccountCellProtectFieldIsModified,
                    "The witness.{} field of inputs[{}] and outputs[{}] should be the same.",
                    $field_name,
                    input_index,
                    output_index
                );
            )*
        };
    }

    macro_rules! das_assert_field_consistent_if_not_except {
        ($input_witness_reader:expr, $output_witness_reader:expr, $( ($field:ident, $field_name:expr) ),*) => {
            $(
                if !except.contains(&$field_name) {
                    das_assert_field_consistent!(
                        $input_witness_reader,
                        $output_witness_reader,
                        ($field, $field_name)
                    );
                }
            )*
        };
    }

    das_assert_field_consistent!(
        input_witness_reader,
        output_witness_reader,
        (id, "id"),
        (account, "account") //(registered_at, "registered_at")  // warning: mv to "if_not_except"
    );

    das_assert_field_consistent_if_not_except!(
        input_witness_reader,
        output_witness_reader,
        (registered_at, "registered_at"),
        (records, "records"),
        (last_transfer_account_at, "last_transfer_account_at"),
        (last_edit_manager_at, "last_edit_manager_at"),
        (last_edit_records_at, "last_edit_records_at"),
        (status, "status")
    );

    match input_witness_reader.version() {
        0 | 1 => {
            // CAREFUL! The early versions will no longer be supported.
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
        2 => {
            // The output witness should be upgraded to the latest version.
            das_assert!(
                output_witness_reader.version() == 4,
                ErrorCode::UpgradeForWitnessIsRequired,
                "The witness of outputs[{}] should be upgraded to latest version.",
                output_index
            );

            let output_witness_reader = output_witness_reader
                .try_into_latest()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;

            // If field enable_sub_account is excepted, skip verifying their defaults.
            if !except.contains(&"enable_sub_account") {
                das_assert!(
                    u8::from(output_witness_reader.enable_sub_account()) == 0
                        && u64::from(output_witness_reader.renew_sub_account_price()) == 0,
                    ErrorCode::UpgradeDefaultValueOfNewFieldIsError,
                    "outputs[{}] The witness.enable_sub_account and witness.renew_sub_account_price should be 0 by default.",
                    output_index
                );
            }

            if !except.contains(&"approval") {
                das_assert!(
                    util::is_reader_eq(output_witness_reader.approval(), AccountApproval::default().as_reader()),
                    AccountCellErrorCode::AccountCellProtectFieldIsModified,
                    "outputs[{}] The witness.approval should be default value.",
                    output_index
                )
            }
        }
        3 => {
            // The output witness should be upgraded to the latest version.
            das_assert!(
                output_witness_reader.version() == 4,
                ErrorCode::UpgradeForWitnessIsRequired,
                "The witness of outputs[{}] should be upgraded to latest version.",
                output_index
            );

            // Verify if the new fields is consistent.
            let input_witness_reader = input_witness_reader
                .try_into_v3()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;
            let output_witness_reader = output_witness_reader
                .try_into_latest()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;

            das_assert_field_consistent_if_not_except!(
                input_witness_reader,
                output_witness_reader,
                (enable_sub_account, "enable_sub_account"),
                (renew_sub_account_price, "renew_sub_account_price")
            );

            if !except.contains(&"approval") {
                das_assert!(
                    util::is_reader_eq(output_witness_reader.approval(), AccountApproval::default().as_reader()),
                    AccountCellErrorCode::AccountCellProtectFieldIsModified,
                    "outputs[{}] The witness.approval should be default value.",
                    output_index
                )
            }
        }
        _ => {
            // Verify if the new fields is consistent.
            let input_witness_reader = input_witness_reader
                .try_into_latest()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;
            let output_witness_reader = output_witness_reader
                .try_into_latest()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;

            das_assert_field_consistent_if_not_except!(
                input_witness_reader,
                output_witness_reader,
                (enable_sub_account, "enable_sub_account"),
                (renew_sub_account_price, "renew_sub_account_price"),
                (approval, "approval")
            );
        }
    }

    Ok(())
}

pub fn verify_account_witness_record_empty<'a>(
    account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    cell_index: usize,
    source: Source,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if AccountCell.witness.records is empty.");

    if account_cell_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        let records = account_cell_witness_reader.records();

        das_assert!(
            records.len() == 0,
            AccountCellErrorCode::AccountCellRecordNotEmpty,
            "{:?}[{}]The AccountCell.witness.records should be empty.",
            source,
            cell_index
        );
    }

    Ok(())
}

pub fn verify_account_no_other_type_cell_use_das_lock_in_inputs(
    type_id_table: TypeIdTableReader,
) -> Result<(), Box<dyn ScriptError>> {
    let das_lock = das_lock();
    let input_cells_with_das_lock = find_cells_by_script(ScriptType::Lock, das_lock.as_reader().into(), Source::Input)?;
    let account_cell_type_id = type_id_table.account_cell();
    let dp_cell_type_id = type_id_table.dpoint_cell();

    let account_cell_type_id_hash = blake2b_256(account_cell_type_id.as_slice());
    let dp_cell_type_id_hash = blake2b_256(dp_cell_type_id.as_slice());

    for i in input_cells_with_das_lock {
        let cell_type_id = high_level::load_cell_type_hash(i, Source::Input)?;
        if let Some(cell_type_id) = cell_type_id {
            if cell_type_id == account_cell_type_id_hash || cell_type_id == dp_cell_type_id_hash {
                continue;
            } else {
                warn!("The input cell type id is not account cell or dp cell.");
                return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
            }
        } else {
            warn!("The input cell type id is none.");
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
    }
    Ok(())
}

pub fn verify_status_conversion<'a>(
    input_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    expected_input_status: AccountStatus,
    expected_output_status: AccountStatus,
) -> Result<(), Box<dyn ScriptError>> {
    if input_account_cell_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        let input_status = u8::from(input_account_cell_witness_reader.status());
        let output_status = u8::from(output_account_cell_witness_reader.status());

        das_assert!(
            input_status == expected_input_status as u8,
            AccountCellErrorCode::AccountCellStatusLocked,
            "The AccountCell.witness.status should be {:?} in inputs, received {}",
            expected_input_status,
            input_status
        );
        das_assert!(
            output_status == expected_output_status as u8,
            AccountCellErrorCode::AccountCellStatusLocked,
            "The AccountCell.witness.status should be {:?} in outputs, received {}",
            expected_output_status,
            output_status
        );
    }

    Ok(())
}

pub fn verify_status<'a>(
    account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    expected_status: AccountStatus,
    index: usize,
    source: Source,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "{:?}[{}] Verify if AccountCell is in {:?} status.",
        source, index, expected_status
    );

    if account_cell_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        let account_cell_status = u8::from(account_cell_witness_reader.status());

        das_assert!(
            account_cell_status == expected_status as u8,
            AccountCellErrorCode::AccountCellStatusLocked,
            "{:?}[{}] The AccountCell.witness.status should be {:?}.",
            source,
            index,
            expected_status
        );
    }

    Ok(())
}

pub fn verify_status_v2<'a>(
    account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    expected_status: &[AccountStatus],
    index: usize,
    source: Source,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "{:?}[{}] Verify if AccountCell is in {:?} status.",
        source, index, expected_status
    );

    if account_cell_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        let expected_status = expected_status.iter().map(|s| *s as u8).collect::<Vec<_>>();
        let account_cell_status = u8::from(account_cell_witness_reader.status());

        das_assert!(
            expected_status.contains(&account_cell_status),
            AccountCellErrorCode::AccountCellStatusLocked,
            "{:?}[{}] The AccountCell.witness.status should be {:?}.",
            source,
            index,
            expected_status
        );
    }

    Ok(())
}

pub fn verify_sub_account_enabled<'a>(
    account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    index: usize,
    source: Source,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "{:?}[{}] Verify if the AccountCell has enabled sub-account feature.",
        source, index
    );

    if account_cell_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else if account_cell_witness_reader.version() == 2 {
        return Err(code_to_error!(SubAccountCellErrorCode::SubAccountFeatureNotEnabled));
    }

    let enable_sub_account = match account_cell_witness_reader.version() {
        3 => {
            let reader = account_cell_witness_reader
                .try_into_v3()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;
            reader.enable_sub_account()
        }
        _ => {
            let reader = account_cell_witness_reader
                .try_into_latest()
                .map_err(|_| ErrorCode::NarrowMixerTypeFailed)?;
            reader.enable_sub_account()
        }
    };

    das_assert!(
        u8::from(enable_sub_account) == 1,
        SubAccountCellErrorCode::SubAccountFeatureNotEnabled,
        "{:?}[{}]The AccountCell.witness.enable_sub_account should be 1.",
        source,
        index
    );

    Ok(())
}

pub fn verify_account_cell_consistent_with_exception<'a>(
    input_account_cell: usize,
    output_account_cell: usize,
    input_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    changed_lock: Option<&str>,
    except_data: Vec<&str>,
    except_witness: Vec<&str>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if the AccountCell's data & lock & witness is consistent in inputs and outputs.");

    verify_account_lock_consistent(input_account_cell, output_account_cell, changed_lock)?;
    verify_account_data_consistent(input_account_cell, output_account_cell, except_data)?;
    verify_account_witness_consistent(
        input_account_cell,
        output_account_cell,
        &input_account_cell_witness_reader,
        &output_account_cell_witness_reader,
        except_witness,
    )?;

    Ok(())
}

pub fn verify_preserved_accounts(account: &[u8]) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if account is preserved.");

    let account_hash = util::blake2b_256(account);
    let account_id = account_hash.get(..ACCOUNT_ID_LENGTH).unwrap();
    let index = (account_id[0] % PRESERVED_ACCOUNT_CELL_COUNT) as usize;
    let data_type = types_util::preserved_accounts_group_to_data_type(index);
    let preserved_accounts = Config::get_instance().preserved_account(data_type)?;

    // debug!(
    //     "account: {}, account ID: {:?}, data_type: {:?}",
    //     String::from_utf8(account.to_vec()).unwrap(),
    //     account_id,
    //     data_type
    // );

    if util::is_account_id_in_collection(account_id, preserved_accounts) {
        warn!(
            "Account {} is preserved. (hex: 0x{}, hash: 0x{})",
            String::from_utf8(account.to_vec()).unwrap(),
            util::hex_string(account),
            util::hex_string(&account_hash)
        );
        return Err(code_to_error!(ErrorCode::AccountIsPreserved));
    }

    Ok(())
}

/// Verify if the account can never be registered.
pub fn verify_unavailable_accounts(account: &[u8]) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if account if unavailable");

    let account_hash = util::blake2b_256(account);
    let account_id = account_hash.get(..ACCOUNT_ID_LENGTH).unwrap();
    let unavailable_accounts = Config::get_instance().unavailable_account()?;

    if util::is_account_id_in_collection(account_id, unavailable_accounts) {
        warn!(
            "Account {} is unavailable. (hex: 0x{}, hash: 0x{})",
            String::from_utf8(account.to_vec()).unwrap(),
            util::hex_string(account),
            util::hex_string(&account_hash)
        );
        return Err(code_to_error!(ErrorCode::AccountIsUnAvailable));
    }

    Ok(())
}

pub fn verify_account_chars(chars_reader: AccountCharsReader) -> Result<(), Box<dyn ScriptError>> {
    let mut prev_char_set_name: Option<_> = None;
    for account_char in chars_reader.iter() {
        // Loading different charset configs on demand.
        let data_type = types_util::char_set_to_data_type(CharSetType::try_from(account_char.char_set_name()).unwrap());
        let char_set_index = types_util::data_type_to_char_set(data_type) as usize;

        // Check if account contains only one non-global character set.
        match Config::get_instance().char_set(char_set_index) {
            Some(Ok(char_set)) => {
                if !char_set.global {
                    if prev_char_set_name.is_none() {
                        prev_char_set_name = Some(char_set_index);
                    } else {
                        let pre_char_set_index = prev_char_set_name.as_ref().unwrap();
                        das_assert!(
                            pre_char_set_index == &char_set_index,
                            ErrorCode::CharSetIsConflict,
                            "Non-global CharSet[{}] has been used by account, so CharSet[{}] can not be used together.",
                            pre_char_set_index,
                            char_set_index
                        );
                    }
                }
            }
            Some(Err(err)) => {
                return Err(err);
            }
            None => {
                warn!("[1] Chan not found CharSet[{}].", char_set_index);
                return Err(code_to_error!(ErrorCode::CharSetIsUndefined));
            }
        }
    }

    let tmp = vec![0u8];
    let mut required_char_sets = vec![tmp.as_slice(); CHAR_SET_LENGTH];
    for account_char in chars_reader.iter() {
        let char_set_index = u32::from(account_char.char_set_name()) as usize;
        if required_char_sets[char_set_index].len() <= 1 {
            let char_set = match Config::get_instance().char_set(char_set_index) {
                Some(Ok(char_set)) => char_set,
                Some(Err(err)) => {
                    return Err(err);
                }
                None => {
                    warn!("[2] Chan not found CharSet[{}].", char_set_index);
                    return Err(code_to_error!(ErrorCode::CharSetIsUndefined));
                }
            };
            required_char_sets[char_set_index] = char_set.data.as_slice();
        }

        let account_char_bytes = account_char.bytes().raw_data();
        let mut found = false;
        let mut from = 0;
        for (i, item) in required_char_sets[char_set_index].iter().enumerate() {
            if item == &0 {
                let char_bytes = required_char_sets[char_set_index].get(from..i).unwrap();
                if account_char_bytes == char_bytes {
                    found = true;
                    break;
                }

                from = i + 1;
            }
        }

        das_assert!(
            found,
            ErrorCode::AccountCharIsInvalid,
            "The character {}(utf-8: 0x{}) can not be used in account, because it is not contained by CharSet[{}].",
            String::from_utf8(account_char_bytes.to_vec()).unwrap(),
            util::hex_string(account_char.bytes().raw_data()),
            char_set_index
        );
    }

    Ok(())
}

pub fn verify_account_chars_max_length(chars_reader: AccountCharsReader) -> Result<(), Box<dyn ScriptError>> {
    let config = Config::get_instance().account()?;
    let max_chars_length = u32::from(config.max_length());
    let account_chars_length = chars_reader.len() as u32;

    das_assert!(
        max_chars_length >= account_chars_length,
        ErrorCode::AccountIsTooLong,
        "The maximum length of account is {}, but {} found.",
        max_chars_length,
        account_chars_length
    );

    Ok(())
}

pub fn verify_account_chars_min_length(chars_reader: AccountCharsReader) -> Result<(), Box<dyn ScriptError>> {
    let minimum_length = 1;
    let account_chars_length = chars_reader.len() as u32;

    das_assert!(
        account_chars_length >= minimum_length,
        ErrorCode::AccountIsTooShort,
        "The minimum length of account is {}, but {} found.",
        minimum_length,
        account_chars_length
    );

    Ok(())
}

pub fn verify_records_keys(records: RecordsReader) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if records keys are available.");

    let config_account = Config::get_instance().account()?;
    let record_key_namespace = Config::get_instance().record_key_namespace()?;
    let records_max_size = u32::from(config_account.record_size_limit()) as usize;

    das_assert!(
        records.total_size() <= records_max_size,
        AccountCellErrorCode::AccountCellRecordSizeTooLarge,
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
        let record_type = Vec::from(record.record_type().raw_data());
        let record_key = Vec::from(record.record_key().raw_data());
        match record_type.as_slice() {
            b"custom_key" => {
                // CAREFUL Triple check
                for char in record_key.iter() {
                    das_assert!(
                        CUSTOM_KEYS_NAMESPACE.contains(char),
                        AccountCellErrorCode::AccountCellRecordKeyInvalid,
                        "The keys in custom_key should only contain digits, lowercase alphabet and underline."
                    );
                }
            }
            _ => {
                let mut record_type_and_key = record_type.clone();
                record_type_and_key.push(46);
                record_type_and_key.extend_from_slice(&record_key);

                let mut is_valid = false;
                for key in &key_list {
                    if vec_compare(record_type_and_key.as_slice(), *key) {
                        is_valid = true;
                        break;
                    }
                }

                // For compatibility, the address records is allowed to use digit chars.
                if record_type == b"address" && !is_valid {
                    for (i, char) in record_key.iter().enumerate() {
                        if i == 0 && record_key.len() > 1 {
                            das_assert!(
                                char != &48, // 48 is the ascii code of '0'
                                AccountCellErrorCode::AccountCellRecordKeyInvalid,
                                "The first char of the key in address should not be '0'."
                            );
                        }

                        das_assert!(
                            COIN_TYPE_DIGITS.contains(char),
                            AccountCellErrorCode::AccountCellRecordKeyInvalid,
                            "The keys in address should only contain digits."
                        );
                    }

                    is_valid = true;
                }

                das_assert!(
                    is_valid,
                    AccountCellErrorCode::AccountCellRecordKeyInvalid,
                    "Account cell record key is invalid: {:?}",
                    String::from_utf8(record_type)
                );
            }
        }
    }

    Ok(())
}
