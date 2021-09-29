use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::{constants::*, packed::*, prelude::*};
use serde_json::json;

fn gen_inviter_and_channel_locks(inviter_args: &str, channel_args: &str) -> (Script, Script) {
    let inviter_lock = gen_fake_das_lock(&gen_das_lock_args(inviter_args, None));
    let channel_lock = gen_fake_das_lock(&gen_das_lock_args(channel_args, None));
    (inviter_lock, channel_lock)
}

fn gen_params(inviter_args: &str, channel_args: &str) -> String {
    let (inviter_lock, channel_lock) = gen_inviter_and_channel_locks(inviter_args, channel_args);

    format!(
        "0x{}{}",
        util::bytes_to_hex(inviter_lock.as_slice()),
        util::bytes_to_hex(channel_lock.as_slice())
    )
}

test_with_generator!(test_account_sale_buy, || {
    let params = gen_params(
        "0x050000000000000000000000000000000000008888",
        "0x050000000000000000000000000000000000009999",
    );
    let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    template.push_cell_v2(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Selling as u8)
            }
        }),
        Source::Input,
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
    template.push_cell_v2(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "xxxxx.bit",
                "price": "20_000_000_000",
                "description": "This is some account description.",
                "started_at": timestamp
            }
        }),
        Source::Input,
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
    template.push_cell_v2(
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "manager_lock_args": "0x050000000000000000000000000000000000002222",
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        Source::Input,
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    // outputs
    template.push_cell_v2(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "manager_lock_args": "0x050000000000000000000000000000000000002222"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Source::Output,
        Some(2),
    );

    template.push_cell_v2(
        json!({
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{income-cell-type}}"
            },
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000008888", None)
                        },
                        "capacity": "200_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000009999", None)
                        },
                        "capacity": "200_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "200_000_000"
                    }
                ]
            }
        }),
        Source::Output,
        None,
    );

    template.push_cell_v2(
        json!({
            "capacity": "40_099_990_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111",
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        Source::Output,
        None,
    );

    template.as_json()
});
