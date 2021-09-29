use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

test_with_generator!(test_account_sale_edit, || {
    let (mut template, timestamp) = init("edit_account_sale", Some("0x00"));

    // inputs
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

    // outputs
    template.push_cell_v2(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "xxxxx.bit",
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        Source::Output,
        None,
    );

    template.as_json()
});
