use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_edit("edit_sub_account", None);

    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );

    template.restore_sub_account(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
    ]);

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0);

    template
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_input_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "profit": profit
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "profit": profit
            }
        }),
    );
}

#[test]
fn test_sub_account_edit() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            // Simulate modifying manager.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "owner",
            // Simulate modifying owner.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x01",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
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
    push_simple_output_sub_account_cell(&mut template, 0);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_edit_parent_expired() {
    let mut template = init_edit("edit_sub_account", None);

    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "data": {
                // Simulate the parent AccountCell has been expired.
                "expired_at": TIMESTAMP - 1
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );

    template.restore_sub_account(vec![json!({
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": MANAGER_1
        },
        "account": SUB_ACCOUNT_1,
        "suffix": SUB_ACCOUNT_SUFFIX,
        "registered_at": TIMESTAMP,
        "expired_at": u64::MAX,
    })]);

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0);

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            // Simulate modifying manager.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0);

    challenge_tx(template.as_json(), Error::AccountCellInExpirationGracePeriod)
}

#[test]
fn challenge_sub_account_edit_parent_not_enable_feature() {
    let mut template = init_edit("edit_sub_account", None);

    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                // Simulate the parent AccountCell has not enabled the sub-account feature.
                "enable_sub_account": 0,
            }
        }),
    );

    template.restore_sub_account(vec![json!({
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": MANAGER_1
        },
        "account": SUB_ACCOUNT_1,
        "suffix": SUB_ACCOUNT_SUFFIX,
        "registered_at": TIMESTAMP,
        "expired_at": u64::MAX,
    })]);

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0);

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            // Simulate modifying manager.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0);

    challenge_tx(template.as_json(), Error::SubAccountFeatureNotEnabled)
}

#[test]
fn challenge_sub_account_edit_owner_not_change() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "owner",
            // Simulate owner is not changed when editing it.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_1))
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
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::SubAccountEditLockError)
}

#[test]
fn challenge_sub_account_edit_owner_changed_when_edit_manager() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            // Simulate owner is changed when editing manager.
            "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_2))
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
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::SubAccountEditLockError)
}

#[test]
fn challenge_sub_account_edit_manager_not_change() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            // Simulate manager is not changed when editing.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_1))
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
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::SubAccountEditLockError)
}

#[test]
fn challenge_sub_account_edit_records_invalid_char() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x01",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
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
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellRecordKeyInvalid)
}

#[test]
fn challenge_sub_account_edit_records_invalid_key() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x01",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
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
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellRecordKeyInvalid)
}

#[test]
fn challenge_sub_account_edit_records_invalid_role() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
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
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellPermissionDenied)
}

#[test]
fn challenge_sub_account_profit_changed() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sign_role": "0x00",
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
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
                // Simulate modifying profit of the SubAccountCell.
                "profit": 1
            }
        }),
    );

    challenge_tx(template.as_json(), Error::SubAccountCellConsistencyError)
}
