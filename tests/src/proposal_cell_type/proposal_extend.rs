use super::common::*;
use crate::util::{constants::*, template_common_cell::*, template_generator::TemplateGenerator, template_parser::*};
use das_types_std::constants::*;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("extend_proposal");

    // cell_deps
    push_dep_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00013.bit"
                        },
                    ],
                ]
            }
        }),
    );
    // slices[0]
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00005.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00009.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00002.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000003333",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
    // slices[1]
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00004.bit",
                "next": "das00011.bit"
            },
            "witness": {
                "account": "das00004.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00018.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000004444",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    template
}

#[test]
fn test_proposal_extend() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::Proposed as u8,
                            "next": "das00009.bit"
                        },
                        {
                            "account_id": "das00009.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00002.bit"
                        },
                        {
                            "account_id": "das00002.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00013.bit"
                        },
                    ],
                    [
                        {
                            "account_id": "das00004.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00018.bit"
                        },
                        {
                            "account_id": "das00018.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00011.bit"
                        },
                    ]
                ]
            }
        }),
    );

    test_tx(template.as_json());
}
