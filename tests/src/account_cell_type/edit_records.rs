use super::common::init;
use crate::util::{
    accounts::*, constants::*, error::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("edit_records", Some("0x01"));

    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    push_input_account_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    },
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    },
                    {
                        "type": "address",
                        "key": "btc",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000002222",
                    },
                    {
                        "type": "dweb",
                        "key": "ipfs",
                        "label": "Mars",
                        "value": "0x00000000000000000000",
                    },
                    {
                        "type": "profile",
                        "key": "email",
                        "label": "Company",
                        "value": "0x00000000000000000000",
                    },
                    {
                        "type": "custom_key",
                        "key": "xxxx",
                        "label": "xxxxxx",
                        "value": "0x00000000000000000000",
                    }
                ]
            }
        }),
    );

    template
}

#[test]
fn test_account_edit_records() {
    let mut template = before_each();

    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP,
                "records": [
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    },
                    {
                        "type": "address",
                        "key": "60",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_account_edit_records_multiple_cells() {
    let mut template = before_each();

    // inputs
    // Simulate editing multiple AccountCells in one transaction.
    push_input_account_cell(&mut template, json!({}));

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_account_edit_records_with_other_cells() {
    let mut template = init("edit_records", Some("0x01"));

    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER
            }
        }),
    );
    // Simulate transferring some balance of the user at the same time.
    push_input_balance_cell(&mut template, 100_000_000_000, SENDER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_account_edit_records_invalid_char() {
    let mut template = before_each();

    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP,
                "records": [
                    {
                        "type": "custom_key",
                        // Simulate using invalid char in the key field of a record.
                        "key": "eth+",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellRecordKeyInvalid)
}

#[test]
fn challenge_account_edit_records_invalid_key() {
    let mut template = before_each();

    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP,
                "records": [
                    {
                        "type": "dweb",
                        // Simulate using a key out of namespace.
                        "key": "xxxx",
                        "label": "xxxxx",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellRecordKeyInvalid)
}

#[test]
fn challenge_account_edit_records_invalid_coin_type() {
    let mut template = before_each();

    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP,
                "records": [
                    {
                        "type": "address",
                        // Simulate using a non-digit char in key field.
                        "key": "60a",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellRecordKeyInvalid)
}
