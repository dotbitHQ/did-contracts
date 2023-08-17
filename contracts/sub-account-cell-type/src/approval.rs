use alloc::boxed::Box;
use alloc::string::ToString;

use das_core::constants::{das_lock, DAY_SEC};
use das_core::error::*;
use das_core::{code_to_error, das_assert, debug, util, warn};
use das_types::constants::AccountStatus;
use das_types::mixer::SubAccountReaderMixer;
use das_types::packed::*;
use das_types::prelude::*;
use das_types::prettier::Prettier;

pub fn transfer_approval_create(
    i: usize,
    timestamp: u64,
    sub_account_reader: Box<dyn SubAccountReaderMixer + '_>,
    new_sub_account_reader: SubAccountReader,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "  witnesses[{:>2}] Verify if the AccountApprovalTransfer.approval is not exist ...",
        i
    );

    match sub_account_reader.try_into_latest() {
        Ok(sub_account_reader) => {
            das_assert!(
                util::is_reader_eq(sub_account_reader.approval(), AccountApproval::default().as_reader()),
                SubAccountCellErrorCode::ApprovalExist,
                "  witnesses[{:>2}] The sub-account already has approval.",
                i
            );
        }
        Err(_) => {
            debug!(
                "  witnesses[{:>2}] The sub-account does not have approval, could creating a new one.",
                i
            );
        }
    }

    debug!(
        "  witnesses[{:>2}] Verify if the SubAccount has more than 30 days before expired ...",
        i
    );

    let expired_at = u64::from(sub_account_reader.expired_at());
    das_assert!(
        timestamp + 30 * DAY_SEC < expired_at,
        SubAccountCellErrorCode::AccountHasNearGracePeriod,
        "  witnesses[{:>2}] The sub-account {} should be 30 days before expired.",
        i,
        sub_account_reader.account().as_prettier()
    );

    debug!(
        "  witnesses[{:>2}] Verify if the AccountApprovalTransfer.params is valid ...",
        i
    );

    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();

    let approval_reader = new_sub_account_reader.approval();
    let approval_params =
        AccountApprovalTransfer::from_compatible_slice(approval_reader.params().raw_data()).map_err(|e| {
            warn!(
                "  witnesses[{:>2}] Decoding edit_value.params failed: {}",
                i,
                e.to_string()
            );
            return code_to_error!(SubAccountCellErrorCode::WitnessParsingError);
        })?;
    let approval_params_reader = approval_params.as_reader();
    let platform_lock = approval_params_reader.platform_lock();
    let protected_until = u64::from(approval_params_reader.protected_until());
    let sealed_until = u64::from(approval_params_reader.sealed_until());
    let delay_count_remain = u8::from(approval_params_reader.delay_count_remain());
    let to_lock = approval_params_reader.to_lock();

    let limit_days = 10;

    das_assert!(
        util::is_type_id_equal(platform_lock.into(), das_lock_reader),
        SubAccountCellErrorCode::ApprovalParamsPlatformLockInvalid,
        "  witnesses[{:>2}] The edit_value.params.platform_lock should use das-lock.",
        i
    );

    das_assert!(
        protected_until <= timestamp + DAY_SEC * limit_days,
        SubAccountCellErrorCode::ApprovalParamsProtectedUntilInvalid,
        "  witnesses[{:>2}] The edit_value.params.protected_until should not exceed {} days from current.",
        i,
        limit_days
    );

    das_assert!(
        sealed_until <= protected_until + DAY_SEC * limit_days,
        SubAccountCellErrorCode::ApprovalParamsSealedUntilInvalid,
        "  witnesses[{:>2}] The edit_value.params.sealed_until should not exceed {} days from the protected_until datetime.",
        i,
        limit_days
    );

    das_assert!(
        delay_count_remain == 1,
        SubAccountCellErrorCode::ApprovalParamsDelayCountRemainInvalid,
        "  witnesses[{:>2}] The edit_value.params.delay_count_remain should be 1.",
        i
    );

    das_assert!(
        util::is_type_id_equal(to_lock.into(), das_lock_reader),
        SubAccountCellErrorCode::ApprovalParamsToLockInvalid,
        "  witnesses[{:>2}] The edit_value.params.to_lock should use das-lock.",
        i
    );

    Ok(())
}

pub fn transfer_approval_delay(
    i: usize,
    prev_approval_reader: AccountApprovalReader,
    current_approval_reader: AccountApprovalReader,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("  witnesses[{:>2}] Verify if the approval is consistant ...", i);

    das_assert!(
        util::is_reader_eq(prev_approval_reader.action(), current_approval_reader.action()),
        SubAccountCellErrorCode::ApprovalParamsCanNotBeChanged,
        "  witnesses[{:>2}] The edit_value.action can not be changed.",
        i
    );

    let prev_approval_params = AccountApprovalTransfer::from_compatible_slice(prev_approval_reader.params().raw_data())
        .map_err(|e| {
            warn!(
                "  witnesses[{:>2}] Decoding sub_account.approval.params failed: {}",
                i,
                e.to_string()
            );
            return code_to_error!(SubAccountCellErrorCode::WitnessParsingError);
        })?;
    let prev_approval_params_reader = prev_approval_params.as_reader();
    let current_approval_params =
        AccountApprovalTransfer::from_compatible_slice(current_approval_reader.params().raw_data()).map_err(|e| {
            warn!(
                "  witnesses[{:>2}] Decoding edit_value.params failed: {}",
                i,
                e.to_string()
            );
            return code_to_error!(SubAccountCellErrorCode::WitnessParsingError);
        })?;
    let current_approval_params_reader = current_approval_params.as_reader();

    macro_rules! das_assert_field_consistent {
        ($prev_reader:expr, $current_reader:expr, $field_name:expr, $field:ident) => {
            das_assert!(
                util::is_reader_eq($prev_reader.$field(), $current_reader.$field()),
                SubAccountCellErrorCode::ApprovalParamsCanNotBeChanged,
                "  witnesses[{:>2}] The edit_value.params.{} can not be changed.",
                i,
                $field_name
            );
        };
    }

    das_assert_field_consistent!(
        prev_approval_params_reader,
        current_approval_params_reader,
        "platform_lock",
        platform_lock
    );
    das_assert_field_consistent!(
        prev_approval_params_reader,
        current_approval_params_reader,
        "protected_until",
        protected_until
    );
    das_assert_field_consistent!(
        prev_approval_params_reader,
        current_approval_params_reader,
        "to_lock",
        to_lock
    );

    let prev_delay_count_remain = u8::from(prev_approval_params_reader.delay_count_remain());
    let current_delay_count_remain = u8::from(current_approval_params_reader.delay_count_remain());

    das_assert!(
        prev_delay_count_remain > 0,
        SubAccountCellErrorCode::ApprovalParamsDelayCountNotEnough,
        "  witnesses[{:>2}] The edit_value.params.delay_count_remain should > 0.",
        i
    );

    das_assert!(
        current_delay_count_remain < prev_delay_count_remain
            && current_delay_count_remain == prev_delay_count_remain - 1,
        SubAccountCellErrorCode::ApprovalParamsDelayCountDecrementError,
        "  witnesses[{:>2}] The edit_value.params.delay_count_remain should be decreased by 1.",
        i
    );

    let prev_sealed_until = u64::from(prev_approval_params_reader.sealed_until());
    let current_sealed_until = u64::from(current_approval_params_reader.sealed_until());

    das_assert!(
        current_sealed_until > prev_sealed_until,
        SubAccountCellErrorCode::ApprovalParamsSealedUntilIncrementError,
        "  witnesses[{:>2}] The edit_value.params.sealed_until should be increased.",
        i
    );

    Ok(())
}

pub fn transfer_approval_revoke(
    i: usize,
    timestamp: u64,
    prev_approval_reader: AccountApprovalReader,
    new_sub_account_reader: SubAccountReader,
) -> Result<(), Box<dyn ScriptError>> {
    debug!(
        "  witnesses[{:>2}] Verify if the approval has exited the protection period ...",
        i
    );

    let prev_approval_params = AccountApprovalTransfer::from_compatible_slice(prev_approval_reader.params().raw_data())
        .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessParsingError))?;
    let protected_until = u64::from(prev_approval_params.protected_until());

    das_assert!(
        timestamp > protected_until,
        SubAccountCellErrorCode::ApprovalInProtectionPeriod,
        "  witnesses[{:>2}] The approval is in protection period, which will end at {}.",
        i,
        protected_until
    );

    debug!("  witnesses[{:>2}] Verify if the approval is revoked ...", i);

    das_assert!(
        u8::from(new_sub_account_reader.status()) == AccountStatus::Normal as u8,
        SubAccountCellErrorCode::AccountStatusError,
        "  witnesses[{:>2}] The sub_account.status should be Normal.",
        i
    );

    das_assert!(
        util::is_reader_eq(
            new_sub_account_reader.approval(),
            AccountApproval::default().as_reader()
        ),
        SubAccountCellErrorCode::ApprovalNotRevoked,
        "  witnesses[{:>2}] The sub_account.approval should be revoked.",
        i
    );

    Ok(())
}
