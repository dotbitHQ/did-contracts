use super::{constants::*, template_generator::TemplateGenerator, util};
use das_types::constants::AccountStatus;
use serde_json::{json, Value};

pub fn push_dep_pre_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{pre-account-cell-type}}"
        },
        "witness": {
            "account": "xxxxx.bit",
            "refund_lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": "0x0000000000000000000000000000000000001111"
            },
            "owner_lock_args": "0x050000000000000000000000000000000000001111050000000000000000000000000000000000001111",
            "inviter_id": Value::Null,
            "inviter_lock": Value::Null,
            "channel_lock": Value::Null,
            "price": {
                "length": 5,
                "new": ACCOUNT_PRICE_5_CHAR,
                "renew": ACCOUNT_PRICE_5_CHAR
            },
            "quote": CKB_QUOTE,
            "invited_discount": INVITED_DISCOUNT,
            "created_at": Value::Null
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_dep(cell, None);
}

pub fn push_input_pre_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_register_fee(5, false),
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{pre-account-cell-type}}"
        },
        "witness": {
            "account": "xxxxx.bit",
            "refund_lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": "0x0000000000000000000000000000000000001111"
            },
            "owner_lock_args": "0x050000000000000000000000000000000000001111050000000000000000000000000000000000001111",
            "inviter_id": Value::Null,
            "inviter_lock": Value::Null,
            "channel_lock": Value::Null,
            "price": {
                "length": 5,
                "new": ACCOUNT_PRICE_5_CHAR,
                "renew": ACCOUNT_PRICE_5_CHAR
            },
            "quote": CKB_QUOTE,
            "invited_discount": INVITED_DISCOUNT,
            "created_at": Value::Null
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None);
}

pub fn push_dep_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": "0x000000000000000000000000000000000000001111",
            "manager_lock_args": "0x000000000000000000000000000000000000001111"
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": "xxxxx.bit",
            "next": "yyyyy.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": "xxxxx.bit",
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_dep(cell, Some(2));
}

pub fn push_input_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": "0x000000000000000000000000000000000000001111",
            "manager_lock_args": "0x000000000000000000000000000000000000001111"
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": "xxxxx.bit",
            "next": "yyyyy.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": "xxxxx.bit",
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(2));
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_account_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": "0x000000000000000000000000000000000000001111",
            "manager_lock_args": "0x000000000000000000000000000000000000001111"
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": "xxxxx.bit",
            "next": "yyyyy.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": "xxxxx.bit",
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, Some(2));
}

pub fn push_input_income_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{income-cell-type}}"
        },
        "witness": {
            "creator": {
                "code_hash": "{{fake-das-lock}}",
                "args": COMMON_INCOME_CREATOR_LOCK_ARGS
            },
            "records": []
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None);
    template.push_empty_witness();
}

pub fn push_output_income_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{income-cell-type}}"
        },
        "witness": {
            "creator": {
                "code_hash": "{{fake-das-lock}}",
                "args": COMMON_INCOME_CREATOR_LOCK_ARGS
            },
            "records": []
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, None);
}

pub fn push_input_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
}

pub fn push_input_normal_cell(template: &mut TemplateGenerator, capacity: u64, args: &str) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": args
            }
        }),
        None,
    );
    template.push_empty_witness();
}

pub fn push_output_normal_cell(template: &mut TemplateGenerator, capacity: u64, args: &str) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": args
            }
        }),
        None,
    );
}
