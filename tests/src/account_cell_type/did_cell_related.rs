use ::did_cell_type::error::ErrorCode as DidCellErrorCode;
use ckb_hash::Blake2bBuilder;
use ckb_types::packed::CellInputBuilder;
use ckb_types::prelude::Entity;
use das_types::constants::{DataType, Source};
use molecule::prelude::Builder;
use serde_json::json;

use super::common::init;
use super::renew_account::push_simple_output_income_cell;
use crate::did_cell_type;
use crate::util::accounts::{ACCOUNT_1, OWNER, SENDER};
use crate::util::constants::{OracleCellType, TIMESTAMP};
use crate::util::template_common_cell::*;
use crate::util::template_generator::{ContractType, TemplateGenerator};
use crate::util::template_parser::{challenge_tx, test_tx, TemplateParser};

pub fn calc_type_id(tx_first_input: &[u8], output_index: usize) -> [u8; 32] {
    let mut blake2b = Blake2bBuilder::new(32).personal(b"ckb-default-hash").build();
    blake2b.update(tx_first_input);
    blake2b.update(&(output_index as u64).to_le_bytes());
    let mut verify_id = [0; 32];
    blake2b.finalize(&mut verify_id);
    verify_id
}

pub fn get_type_id_from_template(template: &TemplateGenerator, output_index: usize) -> [u8; 32] {
    let mut parser = TemplateParser::from_data(template.as_json(), 350_000_000);
    parser.try_parse().unwrap();
    let tx = parser.gen_resolved_tx().unwrap();
    let first_input = CellInputBuilder::default()
        .previous_output(tx.resolved_inputs[0].out_point.clone())
        .since(Default::default())
        .build();

    let type_args = calc_type_id(first_input.as_slice(), output_index);
    type_args
}

#[test]
fn test_transfer_and_upgrade() {
    let mut template = init("transfer_account", Some("0x00"));
    template.push_contract_cell("did-cell-type", ContractType::Contract);
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );

    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "last_transfer_account_at": TIMESTAMP,
                "status": 0x99
            }
        }),
    );

    let type_args = get_type_id_from_template(&template, 1);

    did_cell_type::tmpl::push_did_cell_with_args(
        &mut template,
        ACCOUNT_1.as_bytes(),
        u64::MAX,
        0,
        Source::Output,
        type_args,
    );

    test_tx(template.as_json())
}

#[test]
fn test_edit_records_and_upgrade() {
    let mut template = init("edit_records", Some("0x01"));
    template.push_contract_cell("did-cell-type", ContractType::Contract);

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
                ],
                "status": 0x99
            }
        }),
    );

    let type_args = get_type_id_from_template(&template, 1);

    did_cell_type::tmpl::push_did_cell_with_args(
        &mut template,
        ACCOUNT_1.as_bytes(),
        u64::MAX,
        0,
        Source::Output,
        type_args,
    );

    test_tx(template.as_json())
}

#[test]
fn test_renew_and_upgrade() {
    let mut template = init("renew_account", None);
    template.push_contract_cell("income-cell-type", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Quote, 1000);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);
    template.push_contract_cell("did-cell-type", ContractType::Contract);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": TIMESTAMP
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, OWNER);
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);
    let type_args = get_type_id_from_template(&template, 3);

    did_cell_type::tmpl::push_did_cell_with_args(
        &mut template,
        ACCOUNT_1.as_bytes(),
        TIMESTAMP + 31_536_000,
        0,
        Source::Output,
        type_args,
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_renew_and_upgrade_with_inconsistent_expire_at() {
    let mut template = init("renew_account", None);
    template.push_contract_cell("income-cell-type", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Quote, 1000);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);
    template.push_contract_cell("did-cell-type", ContractType::Contract);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": TIMESTAMP
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, OWNER);
    // NOTICE!!!: The witness order matters for witness parser v1. Must push account cell first
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    let type_args = get_type_id_from_template(&template, 3);

    did_cell_type::tmpl::push_did_cell_with_args(
        &mut template,
        ACCOUNT_1.as_bytes(),
        TIMESTAMP + 31_536_001,
        0,
        Source::Output,
        type_args,
    );

    challenge_tx(template.as_json(), DidCellErrorCode::ExpireAtNotEqual)
}
