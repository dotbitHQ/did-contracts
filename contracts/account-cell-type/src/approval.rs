use alloc::boxed::Box;
use alloc::string::{String, ToString};

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::DAY_SEC;
use das_core::error::*;
use das_core::{code_to_error, das_assert, data_parser, debug, util, verifiers, warn};
use das_types::constants::*;
use das_types::mixer::AccountCellDataReaderMixer;
use das_types::packed::*;
use das_types::prelude::*;
use das_types::prettier::Prettier;

pub fn transfer_approval_create<'a>(
    timestamp: u64,
    input_account_index: usize,
    output_account_index: usize,
    input_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    verifiers::account_cell::verify_status(
        &input_account_reader,
        AccountStatus::Normal,
        input_account_index,
        Source::Input,
    )?;

    verifiers::account_cell::verify_status(
        &output_account_reader,
        AccountStatus::ApprovedTransfer,
        output_account_index,
        Source::Output,
    )?;

    debug!(
        "{:?}[{}] Verify if the AccountApprovalTransfer.approval is not exist ...",
        Source::Input,
        input_account_index
    );

    let no_approval = match input_account_reader.version() {
        4 => {
            let reader = input_account_reader.try_into_latest().unwrap();
            util::is_reader_eq(reader.approval(), AccountApproval::default().as_reader())
        }
        _ => true,
    };
    das_assert!(
        no_approval,
        AccountCellErrorCode::ApprovalExist,
        "{:?}[{}] The account already has approval.",
        Source::Input,
        input_account_index
    );

    debug!("Verify if the Account has more than 30 days before expired ...",);

    let data = util::load_cell_data(output_account_index, Source::Output)?;
    let expired_at = data_parser::account_cell::get_expired_at(data.as_slice());
    das_assert!(
        timestamp + 30 * DAY_SEC < expired_at,
        AccountCellErrorCode::AccountHasNearGracePeriod,
        "The account {} should be 30 days before expired.",
        output_account_reader.account().as_prettier()
    );

    debug!("Verify if the AccountApprovalTransfer.params is valid ...",);

    let output_account_reader = match output_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Output,
                output_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };

    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let approval_reader = output_account_reader.approval();
    let approval_params =
        AccountApprovalTransfer::from_compatible_slice(approval_reader.params().raw_data()).map_err(|e| {
            warn!(
                "{:?}[{}] Decoding approval.params failed: {}",
                Source::Output,
                output_account_index,
                e.to_string()
            );
            return code_to_error!(AccountCellErrorCode::WitnessParsingError);
        })?;
    let approval_params_reader = approval_params.as_reader();
    let platform_lock = approval_params_reader.platform_lock();
    let protected_until = u64::from(approval_params_reader.protected_until());
    let sealed_until = u64::from(approval_params_reader.sealed_until());
    let delay_count_remain = u8::from(approval_params_reader.delay_count_remain());
    let to_lock = approval_params_reader.to_lock();

    let limit_days = 10;

    das_assert!(
        util::is_type_id_equal(platform_lock.into(), das_lock_reader.into()),
        AccountCellErrorCode::ApprovalParamsPlatformLockInvalid,
        "{:?}[{}] The approval.params.platform_lock should use das-lock.",
        Source::Output,
        output_account_index
    );

    das_assert!(
        data_parser::das_lock_args::get_owner_type(platform_lock.args().raw_data()) == (DasLockType::ETH as u8),
        AccountCellErrorCode::ApprovalParamsPlatformLockInvalid,
        "{:?}[{}] The approval.params.platform_lock only support ETH type.",
        Source::Output,
        output_account_index
    );

    das_assert!(
        protected_until <= timestamp + DAY_SEC * limit_days,
        AccountCellErrorCode::ApprovalParamsProtectedUntilInvalid,
        "{:?}[{}] The approval.params.protected_until should not exceed {} days from current.",
        Source::Output,
        output_account_index,
        limit_days
    );

    das_assert!(
        sealed_until <= protected_until + DAY_SEC * limit_days,
        AccountCellErrorCode::ApprovalParamsSealedUntilInvalid,
        "{:?}[{}] The approval.params.sealed_until should not exceed {} days from the protected_until datetime.",
        Source::Output,
        output_account_index,
        limit_days
    );

    das_assert!(
        delay_count_remain == 1,
        AccountCellErrorCode::ApprovalParamsDelayCountRemainInvalid,
        "{:?}[{}] The approval.params.delay_count_remain should be 1.",
        Source::Output,
        output_account_index
    );

    das_assert!(
        util::is_type_id_equal(to_lock.into(), das_lock_reader.into()),
        AccountCellErrorCode::ApprovalParamsToLockInvalid,
        "{:?}[{}] The approval.params.to_lock should use das-lock.",
        Source::Output,
        output_account_index
    );

    Ok(())
}

pub fn transfer_approval_delay<'a>(
    input_account_index: usize,
    output_account_index: usize,
    input_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Parsing the AccountCellData into the latest version ...");

    let input_account_reader = match input_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Input,
                input_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };
    let output_account_reader = match output_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Output,
                output_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };

    debug!("Verify if the AccountApprovalTransfer.params is consistent ...");

    das_assert!(
        util::is_reader_eq(
            input_account_reader.approval().action(),
            output_account_reader.approval().action()
        ),
        AccountCellErrorCode::ApprovalParamsCanNotBeChanged,
        "The AccountCell.witness.approval.action can not be changed.(input: {:?}, output: {:?})",
        String::from_utf8(input_account_reader.approval().action().raw_data().to_vec()),
        String::from_utf8(output_account_reader.approval().action().raw_data().to_vec())
    );

    let input_approval_params = AccountApprovalTransfer::from_compatible_slice(
        input_account_reader.approval().params().raw_data(),
    )
    .map_err(|e| {
        warn!(
            "{:?}[{}] Decoding AccountCell.witness.approval.params failed: {}",
            Source::Input,
            input_account_index,
            e.to_string()
        );
        return code_to_error!(AccountCellErrorCode::WitnessParsingError);
    })?;
    let input_approval_reader = input_approval_params.as_reader();
    let output_approval_params = AccountApprovalTransfer::from_compatible_slice(
        output_account_reader.approval().params().raw_data(),
    )
    .map_err(|e| {
        warn!(
            "{:?}[{}] Decoding AccountCell.witness.approval.params failed: {}",
            Source::Output,
            output_account_index,
            e.to_string()
        );
        return code_to_error!(AccountCellErrorCode::WitnessParsingError);
    })?;
    let output_approval_reader = output_approval_params.as_reader();

    macro_rules! das_assert_field_consistent {
        ($prev_reader:expr, $current_reader:expr, $field_name:expr, $field:ident) => {
            das_assert!(
                util::is_reader_eq($prev_reader.$field(), $current_reader.$field()),
                AccountCellErrorCode::ApprovalParamsCanNotBeChanged,
                "The edit_value.params.{} can not be changed.",
                $field_name
            );
        };
    }
    das_assert_field_consistent!(
        input_approval_reader,
        output_approval_reader,
        "platform_lock",
        platform_lock
    );
    das_assert_field_consistent!(
        input_approval_reader,
        output_approval_reader,
        "protected_until",
        protected_until
    );
    das_assert_field_consistent!(input_approval_reader, output_approval_reader, "to_lock", to_lock);

    debug!("Verify if the AccountApprovalTransfer.params is valid ...");

    let input_delay_count_remain = u8::from(input_approval_reader.delay_count_remain());
    let output_delay_count_remain = u8::from(output_approval_reader.delay_count_remain());

    das_assert!(
        input_delay_count_remain > 0,
        AccountCellErrorCode::ApprovalParamsDelayCountNotEnough,
        "{:?}[{}] The AccountCell.witness.approval.params.delay_count_remain should > 0.",
        Source::Input,
        input_account_index
    );

    das_assert!(
        output_delay_count_remain < input_delay_count_remain
            && output_delay_count_remain == input_delay_count_remain - 1,
        AccountCellErrorCode::ApprovalParamsDelayCountDecrementError,
        "{:?}[{}] The AccountCell.witness.approval.params.delay_count_remain should be decreased by 1.",
        Source::Output,
        output_account_index
    );

    let input_sealed_until = u64::from(input_approval_reader.sealed_until());
    let output_sealed_until = u64::from(output_approval_reader.sealed_until());
    let limit_days = 10;

    das_assert!(
        output_sealed_until > input_sealed_until && output_sealed_until <= (input_sealed_until + DAY_SEC * limit_days),
        AccountCellErrorCode::ApprovalParamsSealedUntilIncrementError,
        "{:?}[{}] The AccountCell.witness.approval.params.sealed_until should be increased properly.({} < sealed_until <= {})",
        Source::Output,
        output_account_index,
        input_sealed_until,
        input_sealed_until + DAY_SEC * limit_days
    );

    Ok(())
}

pub fn transfer_approval_revoke<'a>(
    timestamp: u64,
    input_account_index: usize,
    output_account_index: usize,
    input_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Parsing the AccountCellData into the latest version ...");

    let input_account_reader = match input_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Input,
                input_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };
    let output_account_reader = match output_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Output,
                output_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };

    debug!("Verify if the approval can be revoked ...");

    let input_approval_params = AccountApprovalTransfer::from_compatible_slice(
        input_account_reader.approval().params().raw_data(),
    )
    .map_err(|e| {
        warn!(
            "{:?}[{}] Decoding AccountCell.witness.approval.params failed: {}",
            Source::Input,
            input_account_index,
            e.to_string()
        );
        return code_to_error!(AccountCellErrorCode::WitnessParsingError);
    })?;
    let input_approval_reader = input_approval_params.as_reader();

    let input_protected_until = u64::from(input_approval_reader.protected_until());

    das_assert!(
        timestamp > input_protected_until,
        AccountCellErrorCode::ApprovalInProtectionPeriod,
        "{:?}[{}] The AccountCell.witness.approval.params.protected_until is not reached, can not revoke the approval.",
        Source::Input,
        input_account_index
    );

    // Signature verification is placed at the end of the contract.

    debug!("Verify if the approval has been revoked ...");

    das_assert!(
        (AccountStatus::Normal as u8) == u8::from(output_account_reader.status()),
        AccountCellErrorCode::ApprovalNotRevoked,
        "{:?}[{}] The AccountCell should be reset to the normal status.",
        Source::Output,
        output_account_index
    );

    das_assert!(
        util::is_reader_eq(output_account_reader.approval(), AccountApproval::default().as_reader()),
        AccountCellErrorCode::ApprovalNotRevoked,
        "{:?}[{}] The AccountCell.witness.approval should be set to default.",
        Source::Output,
        output_account_index
    );

    Ok(())
}

pub fn transfer_approval_fulfill<'a>(
    input_account_index: usize,
    output_account_index: usize,
    input_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
    output_account_reader: Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Parsing the AccountCellData into the latest version ...");

    let input_account_reader = match input_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Input,
                input_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };
    let output_account_reader = match output_account_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!(
                "{:?}[{}] The witness should be the latest version.",
                Source::Output,
                output_account_index
            );
            return Err(code_to_error!(AccountCellErrorCode::WitnessParsingError));
        }
    };

    debug!("Parsing the approval params ...");

    let input_approval_params = AccountApprovalTransfer::from_compatible_slice(
        input_account_reader.approval().params().raw_data(),
    )
    .map_err(|e| {
        warn!(
            "{:?}[{}] Decoding AccountCell.witness.approval.params failed: {}",
            Source::Input,
            input_account_index,
            e.to_string()
        );
        return code_to_error!(AccountCellErrorCode::WitnessParsingError);
    })?;
    let input_approval_reader = input_approval_params.as_reader();
    let to_lock = input_approval_reader.to_lock();

    // Signature verification is placed at the end of the contract.

    debug!("Verify if the approval has been fulfilled ...");

    das_assert!(
        (AccountStatus::Normal as u8) == u8::from(output_account_reader.status()),
        AccountCellErrorCode::ApprovalFulfillError,
        "{:?}[{}] The AccountCell should be reset to the normal status.",
        Source::Output,
        output_account_index
    );

    das_assert!(
        util::is_reader_eq(output_account_reader.approval(), AccountApproval::default().as_reader()),
        AccountCellErrorCode::ApprovalFulfillError,
        "{:?}[{}] The AccountCell.witness.approval should be set to default.",
        Source::Output,
        output_account_index
    );

    let output_lock = high_level::load_cell_lock(output_account_index, Source::Output).map_err(|_| {
        warn!(
            "{:?}[{}] Loading lock field failed.",
            Source::Output,
            output_account_index
        );
        return code_to_error!(ErrorCode::InvalidTransactionStructure);
    })?;

    das_assert!(
        util::is_reader_eq(output_lock.as_reader().into(), to_lock),
        AccountCellErrorCode::ApprovalFulfillError,
        "{:?}[{}] The AccountCell.lock should be the to_lock in the approval.",
        Source::Output,
        output_account_index
    );

    das_assert!(
        output_account_reader.records().is_empty(),
        AccountCellErrorCode::ApprovalFulfillError,
        "{:?}[{}] The AccountCell.records should be empty, because the ownership has been changed.",
        Source::Output,
        output_account_index
    );

    Ok(())
}
