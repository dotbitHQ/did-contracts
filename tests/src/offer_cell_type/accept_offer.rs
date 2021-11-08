use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, timestamp: u64, seller: &str, account: &str) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": seller,
                "manager_lock_args": seller
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": account,
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": account,
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_cell(template: &mut TemplateGenerator, timestamp: u64, buyer: &str, account: &str) {
    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": buyer,
                "manager_lock_args": buyer
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": account,
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": account,
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_income_cell(template: &mut TemplateGenerator) {
    template.push_output(
        json!({
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{income-cell-type}}"
            },
            "witness": {
                "records": [
                    // It is a conversion in this transaction that the first record always belong to the creator of the IncomeCell.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "2_000_000_000"
                    }
                ]
            }
        }),
        None,
    );
}

fn before_each(account: &str) -> (TemplateGenerator, u64, &'static str, &'static str) {
    let (mut template, timestamp) = init_with_timestamp("accept_offer");
    let buyer = "0x050000000000000000000000000000000000001111";
    let seller = "0x050000000000000000000000000000000000002222";

    // inputs
    push_input_offer_cell(
        &mut template,
        200_100_000_000,
        buyer,
        account,
        200_000_000_000,
        "Take my money.🍀",
    );
    push_input_account_cell(&mut template, timestamp, seller, account);
    // Transaction builder's BalanceCell
    push_input_balance_cell(
        &mut template,
        100_000_000_000,
        "0x050000000000000000000000000000000000003333",
    );

    (template, timestamp, buyer, seller)
}

test_with_generator!(test_offer_accept_offer, || {
    let account = "xxxxx.bit";
    let (mut template, timestamp, buyer, seller) = before_each(account);

    push_output_account_cell(&mut template, timestamp, buyer, account);
    push_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000, seller);

    template.as_json()
});
