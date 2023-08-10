use das_types_std::constants::*;
use serde_json::{json, Value};

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    template
}

fn push_simple_sub_account_witness(template: &mut TemplateGenerator, sub_account_partial: Value) {
    let mut sub_account = json!({
        "action": SubAccountAction::Edit.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "sub_account": {
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
    });
    util::merge_json(&mut sub_account, sub_account_partial);

    template.push_sub_account_witness_v2(sub_account);
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    push_input_sub_account_cell_v2(
        template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": SubAccountConfigFlag::CustomScript as u8,
            }
        }),
        ACCOUNT_1,
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    push_output_sub_account_cell_v2(
        template,
        json!({
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": SubAccountConfigFlag::CustomScript as u8,
            }
        }),
        ACCOUNT_1,
    );
}

#[test]
fn test_sub_account_edit() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "manager",
            // Simulate modifying manager.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_2,
            },
            "edit_key": "owner",
            // Simulate modifying owner.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sign_role": "0x01",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
            },
            "edit_key": "records",
            // Simulate modifying records.
            "edit_value": [
                {
                    "type": "address",
                    "key": "eth",
                    "label": "Personal",
                    "value": "0x0000000000000000000000000000000000000000",
                },
            ]
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_edit_owner_not_change() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "owner",
            // Simulate owner is not changed when editing it.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountEditLockError);
}

#[test]
fn challenge_sub_account_edit_owner_changed_when_edit_manager() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "manager",
            // Simulate owner is changed when editing manager.
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_2))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountEditLockError);
}

/// If the transaction only contains edit action, then the das_profit must be consistent.
#[test]
fn challenge_sub_account_edit_modify_das_profit() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "owner",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 1, 0);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

/// If the transaction only contains edit action, then the owner_profit must be consistent.
#[test]
fn challenge_sub_account_edit_modify_owner_profit() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "owner",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 1);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_edit_spend_balance_cell_1() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account(vec![json!({
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": MANAGER_1
        },
        "account": SUB_ACCOUNT_1,
        "suffix": SUB_ACCOUNT_SUFFIX,
        "registered_at": TIMESTAMP,
        "expired_at": TIMESTAMP + YEAR_SEC,
    })]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    // Simulate spending the BalanceCells of the parent AccountCell owner in a transaction only contains `edit` action.
    push_input_balance_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "owner",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused,
    );
}

#[test]
fn challenge_sub_account_edit_spend_balance_cell_2() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account(vec![json!({
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": MANAGER_1
        },
        "account": SUB_ACCOUNT_1,
        "suffix": SUB_ACCOUNT_SUFFIX,
        "registered_at": TIMESTAMP,
        "expired_at": TIMESTAMP + YEAR_SEC,
    })]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    // Simulate spending the BalanceCells of others in a transaction only contains `edit` action.
    push_input_balance_cell(&mut template, 10_000_000_000, OWNER_4);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "owner",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused,
    );
}

/// If the transaction only contains edit action, then the owner_profit must be consistent.
#[test]
fn challenge_sub_account_custom_script_changed() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "owner",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );

    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": 0,
                "owner_profit": 0,
                "custom_script": "0x01746573742d637573746f6d2d736372697074"
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_edit_manager_not_change() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "manager",
            // Simulate manager is not changed when editing.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountEditLockError);
}

#[test]
fn challenge_sub_account_edit_records_invalid_char() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sign_role": "0x01",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "records",
            "edit_value": [
                {
                    "type": "custom_key",
                    // Simulate using invalid char in the key field of a record.
                    "key": "eth+",
                    "label": "Company",
                    "value": "0x0000000000000000000000000000000000001111",
                }
            ]
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellRecordKeyInvalid);
}

#[test]
fn challenge_sub_account_edit_records_invalid_key() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sign_role": "0x01",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "records",
            "edit_value": [
                {
                    "type": "dweb",
                    // Simulate using a key out of namespace.
                    "key": "xxxx",
                    "label": "xxxxx",
                    "value": "0x0000000000000000000000000000000000001111",
                }
            ]
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellRecordKeyInvalid);
}

#[test]
fn challenge_sub_account_edit_records_invalid_role() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            // Simulate using owner role to edit the records.
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
            "edit_key": "records",
            "edit_value": [
                {
                    "type": "dweb",
                    "key": "ipfs",
                    "label": "xxxxx",
                    "value": "0x0000000000000000000000000000000000001111",
                }
            ]
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellPermissionDenied);
}

#[test]
fn challenge_sub_account_edit_empty_edit_key() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account(vec![json!({
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": MANAGER_1
        },
        "account": SUB_ACCOUNT_1,
        "suffix": SUB_ACCOUNT_SUFFIX,
        "registered_at": TIMESTAMP,
        "expired_at": TIMESTAMP + YEAR_SEC,
    })]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            },
            "edit_key": "",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::WitnessEditKeyInvalid);
}

#[test]
fn challenge_sub_account_edit_has_expired() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account(vec![json!({
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": MANAGER_1
        },
        "account": SUB_ACCOUNT_1,
        "suffix": SUB_ACCOUNT_SUFFIX,
        "registered_at": TIMESTAMP - YEAR_SEC,
        // Simulate modifying the sub-account that has expired.
        "expired_at": TIMESTAMP - 1,
    })]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP - YEAR_SEC,
                "expired_at": TIMESTAMP - 1,
            },
            "edit_key": "owner",
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::AccountHasInGracePeriod);
}
