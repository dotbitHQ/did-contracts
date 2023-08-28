use std::convert::TryFrom;

use das_types_std::packed::*;
use das_types_std::prelude::*;
use serde_json::Value;

use super::super::util;

pub fn to_v2(path: &str, value: &Value) -> AccountCellDataV2 {
    let (
        _account,
        account_chars,
        account_id,
        registered_at,
        last_transfer_account_at,
        last_edit_manager_at,
        last_edit_records_at,
        status,
        records,
    ) = encode_common_fields(path, value);
    AccountCellDataV2::new_builder()
        .id(account_id)
        .account(account_chars)
        .registered_at(registered_at)
        .last_transfer_account_at(last_transfer_account_at)
        .last_edit_manager_at(last_edit_manager_at)
        .last_edit_records_at(last_edit_records_at)
        .status(status)
        .records(records)
        .build()
}

pub fn to_v3(path: &str, value: &Value) -> AccountCellDataV3 {
    let (
        _account,
        account_chars,
        account_id,
        registered_at,
        last_transfer_account_at,
        last_edit_manager_at,
        last_edit_records_at,
        status,
        records,
    ) = encode_common_fields(path, value);
    let (enable_sub_account, renew_sub_account_price) = encode_v3_fields(path, value);

    AccountCellDataV3::new_builder()
        .id(account_id)
        .account(account_chars)
        .registered_at(registered_at)
        .last_transfer_account_at(last_transfer_account_at)
        .last_edit_manager_at(last_edit_manager_at)
        .last_edit_records_at(last_edit_records_at)
        .status(status)
        .records(records)
        .enable_sub_account(enable_sub_account)
        .renew_sub_account_price(renew_sub_account_price)
        .build()
}

pub fn to_latest(path: &str, value: &Value) -> AccountCellData {
    let (
        _account,
        account_chars,
        account_id,
        registered_at,
        last_transfer_account_at,
        last_edit_manager_at,
        last_edit_records_at,
        status,
        records,
    ) = encode_common_fields(path, value);
    let (enable_sub_account, renew_sub_account_price) = encode_v3_fields(path, value);
    let approval = encode_v4_fields(path, value);

    AccountCellData::new_builder()
        .id(account_id)
        .account(account_chars)
        .registered_at(registered_at)
        .last_transfer_account_at(last_transfer_account_at)
        .last_edit_manager_at(last_edit_manager_at)
        .last_edit_records_at(last_edit_records_at)
        .status(status)
        .records(records)
        .enable_sub_account(enable_sub_account)
        .renew_sub_account_price(renew_sub_account_price)
        .approval(approval)
        .build()
}

fn encode_common_fields(
    path: &str,
    value: &Value,
) -> (
    String,
    AccountChars,
    AccountId,
    Uint64,
    Uint64,
    Uint64,
    Uint64,
    Uint8,
    Records,
) {
    let (account, account_chars) =
        util::parse_json_to_account_chars(&format!("{}.{}", path, "account"), &value["account"], None);
    let account_id = if !value["id"].is_null() {
        util::parse_json_str_to_account_id_mol(&format!("{}.{}", path, "id"), &value["id"])
    } else {
        AccountId::try_from(util::account_to_id(&account)).expect("Calculate account ID from account failed")
    };
    let registered_at = Uint64::from(util::parse_json_u64(
        &format!("{}.{}", path, "registered_at"),
        &value["registered_at"],
        None,
    ));
    let last_transfer_account_at = Uint64::from(util::parse_json_u64(
        &format!("{}.{}", path, "last_transfer_account_at"),
        &value["last_transfer_account_at"],
        Some(0),
    ));
    let last_edit_manager_at = Uint64::from(util::parse_json_u64(
        &format!("{}.{}", path, "last_edit_manager_at"),
        &value["last_edit_manager_at"],
        Some(0),
    ));
    let last_edit_records_at = Uint64::from(util::parse_json_u64(
        &format!("{}.{}", path, "last_edit_records_at"),
        &value["last_edit_records_at"],
        Some(0),
    ));
    let status = Uint8::from(util::parse_json_u8(
        &format!("{}.{}", path, "status"),
        &value["status"],
        Some(0),
    ));
    let records = util::parse_json_to_records_mol(&format!("{}.{}", path, "records"), &value["records"]);

    (
        account,
        account_chars,
        account_id,
        registered_at,
        last_transfer_account_at,
        last_edit_manager_at,
        last_edit_records_at,
        status,
        records,
    )
}

fn encode_v3_fields(path: &str, value: &Value) -> (Uint8, Uint64) {
    let enable_sub_account = Uint8::from(util::parse_json_u8(
        &format!("{}.{}", path, "enable_sub_account"),
        &value["enable_sub_account"],
        Some(0),
    ));
    let renew_sub_account_price = Uint64::from(util::parse_json_u64(
        &format!("{}.{}", path, "renew_sub_account_price"),
        &value["renew_sub_account_price"],
        Some(0),
    ));

    (enable_sub_account, renew_sub_account_price)
}

fn encode_v4_fields(path: &str, value: &Value) -> AccountApproval {
    if value["approval"].is_null() {
        return AccountApproval::default();
    }

    let approval_action =
        util::parse_json_str(&format!("{}.{}", path, "approval.action"), &value["approval"]["action"]);
    let approval_params = match approval_action {
        // "transfer" => {
        // This is use for providing invalid action
        _ => {
            let platform_lock = util::parse_json_script_to_mol(
                &format!("{}.{}", path, "approval.params.platform_lock"),
                &value["approval"]["params"]["platform_lock"],
            );
            let protected_until = util::parse_json_u64(
                &format!("{}.{}", path, "approval.params.protected_until"),
                &value["approval"]["params"]["protected_until"],
                None,
            );
            let sealed_until = util::parse_json_u64(
                &format!("{}.{}", path, "approval.params.sealed_until"),
                &value["approval"]["params"]["sealed_until"],
                None,
            );
            let delay_count_remain = util::parse_json_u8(
                &format!("{}.{}", path, "approval.params.delay_count_remain"),
                &value["approval"]["params"]["delay_count_remain"],
                None,
            );
            let to_lock = util::parse_json_script_to_mol(
                &format!("{}.{}", path, "approval.params.to_lock"),
                &value["approval"]["params"]["to_lock"],
            );
            let account_approval_transfer = AccountApprovalTransfer::new_builder()
                .platform_lock(platform_lock)
                .protected_until(Uint64::from(protected_until))
                .sealed_until(Uint64::from(sealed_until))
                .delay_count_remain(Uint8::from(delay_count_remain))
                .to_lock(to_lock)
                .build();
            Bytes::from(account_approval_transfer.as_slice().to_vec())
        }
        // _ => unimplemented!("Not support action: {}", approval_action),
    };
    let approval = AccountApproval::new_builder()
        .action(Bytes::from(approval_action.as_bytes()))
        .params(approval_params)
        .build();

    approval
}
