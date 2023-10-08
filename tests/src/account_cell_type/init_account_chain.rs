use super::common::init;
use crate::util::{template_common_cell::*, template_parser::*};
use das_types::constants::AccountStatus;
use serde_json::json;

#[test]
fn test_account_init_account_chain() {
    let mut template = init("init_account_chain", None);

    push_input_normal_cell(
        &mut template,
        100_000_000_000,
        "0x0000000000000000000000000000000000000000",
    );

    template.push_output(
        json!({
            "capacity": 28_800_000_000u64,
            "lock": {
                "owner_lock_args": "0x0000000000000000000000000000000000000000",
                "manager_lock_args": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "id": "0x0000000000000000000000000000000000000000",
                "next": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "expired_at": u64::MAX,
            },
            "witness": {
                "id": "0x0000000000000000000000000000000000000000",
                "account": "xxxxx.bit",
                "registered_at": 0,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );

    test_tx(template.as_json());
}
