use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use ckb_testtool::context::Context;
use das_types_std::constants::Source;
use serde_json::json;

use super::common::init;

fn before() -> TemplateGenerator {
    let mut template = init("create_income");

    // inputs
    push_input_normal_cell(&mut template, 20_000_000_000, COMMON_INCOME_CREATOR);

    template
}

#[test]
fn test_income_create() {
    let mut template = before();

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                ]
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_income_create_stored_capacity_error() {
    let mut template = before();

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            // Simulate storing more than actually recorded capacity.
            "capacity": "20_000_000_001",
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::IncomeCellCapacityError)
}

#[test]
fn challenge_income_create_recorded_capacity_error() {
    let mut template = before();

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        // Simulate recording more than actually stored capacity.
                        "capacity": "20_000_000_001"
                    },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_income_create_capacity_error() {
    let mut template = before();

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            // Simulate storing more than Config.
            "capacity": INCOME_BASIC_CAPACITY + 1,
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": INCOME_BASIC_CAPACITY + 1
                    },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_income_create_more_than_one_record() {
    let mut template = before();

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            "capacity": "40_000_000_000",
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                    // Simulate creating more than one record which is not allowed.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_income_create_belong_to_error() {
    let mut template = before();

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            // Simulate writing an unknown lock which is not the paid one.
                            "args": COMMON_PROPOSER
                        },
                        "capacity": "20_000_000_000"
                    },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}
