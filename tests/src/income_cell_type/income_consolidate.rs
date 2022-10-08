use das_types_std::constants::{DataType, Source};
use serde_json::json;

use super::common::init;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before() -> TemplateGenerator {
    let mut template = init("consolidate_income");

    template.push_config_cell(DataType::ConfigCellProfitRate, Source::CellDep);

    template
}

fn push_common_inputs(template: &mut TemplateGenerator) {
    push_input_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                ]
            }
        }),
    );
    push_input_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                ]
            }
        }),
    );
}

#[test]
fn test_income_consolidate_need_pad_1() {
    let mut template = before();

    let capacity_of_10 = 20_000_000_000u64;
    let capacity_of_20 = 10_200_000_000u64;

    // inputs
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": capacity_of_10 / 2, // 100 CKB
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": capacity_of_20 - 200_000_000, // 100 CKB
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000030"
                        },
                        "capacity": 100_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000040"
                        },
                        "capacity": 9_900_000_000u64
                    },
                ]
            }
        }),
    );

    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": capacity_of_10 / 2, // 100 CKB
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": capacity_of_20 - 10_000_000_000, // 2 CKB
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000030"
                        },
                        "capacity": 100_000_000u64
                    },
                ]
            }
        }),
    );

    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF is the keeper who pushed the consolidate_income transaction.
    push_input_normal_cell(
        &mut template,
        6_100_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    // outputs
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 10_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000030"
                        },
                        "capacity": 200_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000040"
                        },
                        "capacity": 9_900_000_000u64
                    },
                ]
            }
        }),
    );
    push_output_normal_cell(&mut template, 40_000_000_000, COMMON_INCOME_CREATOR);
    push_output_normal_cell(
        &mut template,
        9_900_000_000,
        "0x0000000000000000000000000000000000000010",
    );
    push_output_normal_cell(
        &mut template,
        10_098_000_000,
        "0x0000000000000000000000000000000000000020",
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
    push_output_normal_cell(
        &mut template,
        6_300_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    test_tx(template.as_json())
}

#[test]
fn test_income_consolidate_no_pad() {
    let mut template = before();

    // inputs
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 10_000_000_000u64, // 100 CKB
                    },
                ]
            }
        }),
    );
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 200_000_000, // 2 CKB
                    },
                ]
            }
        }),
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF is the keeper who pushed the consolidate_income transaction.
    push_input_normal_cell(
        &mut template,
        6_100_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    // outputs
    push_output_normal_cell(&mut template, 40_000_000_000, COMMON_INCOME_CREATOR);
    push_output_normal_cell(
        &mut template,
        10_098_000_000,
        "0x0000000000000000000000000000000000000010",
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
    push_output_normal_cell(
        &mut template,
        6_162_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    test_tx(template.as_json())
}

#[test]
fn test_income_consolidate_free_fee() {
    let mut template = before();

    // inputs
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 10_000_000_000u64, // 100 CKB
                    },
                ]
            }
        }),
    );
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 10_000_000_000u64, // 100 CKB
                    },
                ]
            }
        }),
    );

    // outputs
    push_output_normal_cell(&mut template, 40_000_000_000, COMMON_INCOME_CREATOR);
    // DAS should be free from consolidating fee.
    push_output_normal_cell(&mut template, 20_000_000_000, DAS_WALLET_LOCK_ARGS);

    test_tx(template.as_json())
}

#[test]
fn test_income_consolidate_big_capacity() {
    let mut template = before();

    let capacity_of_10 = 1_000_000_000_000_000_000u64; // 10 billion CKB

    // inputs
    push_input_income_cell_no_creator(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": capacity_of_10,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": 500_000_000,
                    },
                ]
            }
        }),
    );
    push_input_income_cell_no_creator(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": capacity_of_10,
                    },
                ]
            }
        }),
    );

    // outputs
    push_output_income_cell_no_creator(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 19_500_000_000u64, // 195 CKB
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": 500_000_000,
                    },
                ]
            }
        }),
    );
    push_output_normal_cell(
        &mut template,
        (capacity_of_10 + capacity_of_10 - 19_500_000_000u64) / RATE_BASE * (RATE_BASE - CONSOLIDATING_FEE),
        "0x0000000000000000000000000000000000000010",
    );
    push_output_normal_cell(
        &mut template,
        20_000_000_000_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_income_consolidate_newly_created() {
    let mut template = before();

    // inputs
    // This IncomeCell only contains one record of the creator, it should not be consolidated.
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                ]
            }
        }),
    );
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000030"
                        },
                        "capacity": 500_000_000u64,
                    },
                ]
            }
        }),
    );

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
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000030"
                        },
                        "capacity": 500_000_000u64,
                    },
                ]
            }
        }),
    );
    push_output_normal_cell(&mut template, 20_000_000_000, COMMON_INCOME_CREATOR);

    challenge_tx(
        template.as_json(),
        ErrorCode::IncomeCellConsolidateConditionNotSatisfied,
    );
}

#[test]
fn challenge_income_consolidate_redundant_records() {
    let mut template = before();

    // inputs
    push_common_inputs(&mut template);

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
                        "capacity": 40_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                    // Simulate redundant records for the same lock.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::IncomeCellConsolidateError);
}

#[test]
fn challenge_income_consolidate_redundant_cells() {
    let mut template = before();

    // inputs
    push_common_inputs(&mut template);

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
                        "capacity": 0_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                ]
            }
        }),
    );
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
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 500_000_000u64,
                    },
                ]
            }
        }),
    );
    // Simulate creating extra IncomeCell when consolidating.
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
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 500_000_000u64,
                    },
                ]
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        ErrorCode::IncomeCellConsolidateConditionNotSatisfied,
    );
}

#[test]
fn challenge_income_consolidate_missing_some_records() {
    let mut template = before();

    // inputs
    push_common_inputs(&mut template);

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
                        "capacity": 40_000_000_000u64,
                    },
                    // Simulate missing some records in outputs.
                    // {
                    //     "belong_to": {
                    //         "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    //         "args": "0x0000000000000000000000000000000000000010"
                    //     },
                    //     "capacity": 2_000_000_000u64,
                    // },
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::IncomeCellConsolidateError);
}

#[test]
fn challenge_income_consolidate_wasted_capacity_1() {
    let mut template = before();

    // inputs
    push_common_inputs(&mut template);

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
                        "capacity": 18_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 2_000_000_000u64,
                    },
                ]
            }
        }),
    );
    // Simulate missing some capacity which should be transferred to COMMON_INCOME_CREATOR.
    push_output_normal_cell(&mut template, 20_000_000_000u64, COMMON_INCOME_CREATOR);

    challenge_tx(template.as_json(), ErrorCode::IncomeCellConsolidateError);
}

#[test]
fn challenge_income_consolidate_wasted_capacity_2() {
    let mut template = before();

    // inputs
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000020"
                        },
                        "capacity": 18_000_000_000u64,
                    },
                ]
            }
        }),
    );
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 1_000_000_000u64,
                    },
                ]
            }
        }),
    );

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
                        "capacity": 18_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 2_000_000_000u64,
                    },
                ]
            }
        }),
    );
    // Simulate missing some capacity which should be transferred to COMMON_INCOME_CREATOR.
    push_output_normal_cell(&mut template, 22_000_000_000u64, COMMON_INCOME_CREATOR);

    challenge_tx(template.as_json(), ErrorCode::IncomeCellConsolidateWaste);
}

#[test]
fn challenge_income_consolidate_wasted_capacity_3() {
    let mut template = before();

    // inputs
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 10_000_000_000u64,
                    },
                ]
            }
        }),
    );
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000010"
                        },
                        "capacity": 10_000_000_000u64,
                    },
                ]
            }
        }),
    );

    // outputs
    // Simulate missing some capacity which should be transferred to COMMON_INCOME_CREATOR.
    push_output_normal_cell(&mut template, 20_000_000_000u64, COMMON_INCOME_CREATOR);
    push_output_normal_cell(
        &mut template,
        20_000_000_000u64,
        "0x0000000000000000000000000000000000000010",
    );

    challenge_tx(template.as_json(), ErrorCode::IncomeCellTransferError);
}

#[test]
fn challenge_income_consolidate_eip712_cells_without_type_script() {
    let mut template = before();

    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);

    // inputs
    push_input_income_cell_no_creator(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000000000", None)
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x030000000000000000000000000000000000000010", None)
                        },
                        "capacity": 10_000_000_000u64,
                    },
                ]
            }
        }),
    );
    push_input_income_cell_no_creator(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000000000", None)
                        },
                        "capacity": 20_000_000_000u64,
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x030000000000000000000000000000000000000010", None)
                        },
                        "capacity": 200_000_000u64,
                    },
                ]
            }
        }),
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF is the keeper who pushed the consolidate_income transaction.
    push_input_normal_cell(
        &mut template,
        6_100_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    // outputs
    template.push_output(
        json!({
            "capacity": 39_600_000_000u64,
            "lock": json!({
                "code_hash": "{{fake-das-lock}}",
                "hash_type": "type",
                "args": gen_das_lock_args("0x050000000000000000000000000000000000000000", None)
            }),
        }),
        None,
    );
    template.push_output(
        json!({
            "capacity": 10_098_000_000u64,
            "lock": json!({
                "code_hash": "{{fake-das-lock}}",
                "hash_type": "type",
                "args": gen_das_lock_args("0x030000000000000000000000000000000000000010", None)
            }),
        }),
        None,
    );
    // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF can take some from user as their profit.
    push_output_normal_cell(
        &mut template,
        6_162_000_000,
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    );

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure);

    // challenge_tx(
    //     template.as_json(),
    //     [
    //         Error::InvalidTransactionStructure,
    //         Error::BalanceCellFoundSomeOutputsLackOfType,
    //     ],
    // );
}
