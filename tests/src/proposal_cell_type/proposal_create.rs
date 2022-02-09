use super::common::*;
use crate::util::{
    constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator, template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("propose");

    // cell_deps
    // slices[0]
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00013.bit"
            },
            "witness": {
                "account": "das00012.bit"
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
fn test_proposal_create() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
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

#[test]
fn test_proposal_exist_account_misunderstand() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00002.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );
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
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00002.bit",
                "next": "das00010.bit"
            },
            "witness": {
                "account": "das00002.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00013.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    // Simulate add two continued slices in the sam proposal.
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00002.bit"
                        },
                    ],
                    [
                        {
                            "account_id": "das00002.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00013.bit"
                        },
                        {
                            "account_id": "das00013.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00010.bit"
                        },
                    ]
                ]
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_proposal_create_slices_miss_match_1() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00009.bit"
                        },
                        {
                            "account_id": "das00009.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00013.bit"
                        }
                        // Simulate missing some items in slices.
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

    challenge_tx(template.as_json(), Error::ProposalSliceRelatedCellMissing)
}

#[test]
fn challenge_proposal_create_slices_miss_match_2() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
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
                    ]
                    // Simulate missing some slices in slices.
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceRelatedCellMissing)
}

#[test]
fn challenge_proposal_create_slices_miss_match_3() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        // Simulate mismatch of some AccountCells' account ID.
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00005.bit",
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

    challenge_tx(template.as_json(), Error::ProposalCellAccountIdError)
}

#[test]
fn challenge_proposal_create_slices_miss_match_4() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
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
                        // Simulate mismatch of some PreAccountCells' account ID.
                        {
                            "account_id": "das00005.bit",
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

    challenge_tx(template.as_json(), Error::ProposalCellAccountIdError)
}

#[test]
fn challenge_proposal_create_slices_miss_match_5() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
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
                            // Simulate mismatch of original next in the last item.
                            "next": "das00010.bit"
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

    challenge_tx(template.as_json(), Error::ProposalSliceNotEndCorrectly)
}

#[test]
fn challenge_proposal_create_slices_miss_match_6() {
    let mut template = before_each();

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00009.bit"
                        },
                        {
                            "account_id": "das00009.bit",
                            // Simulate mismatch of item type.
                            "item_type": ProposalSliceItemType::Exist as u8,
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

    challenge_tx(template.as_json(), Error::ProposalCellTypeError)
}

#[test]
fn challenge_proposal_create_empty_slices() {
    let mut template = init("propose");

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                // Simulate an empty slices.
                "slices": []
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSlicesCanNotBeEmpty)
}

#[test]
fn challenge_proposal_create_empty_slice() {
    let mut template = init("propose");

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                // Simulate an empty slice.
                "slices": [[]]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceMustContainMoreThanOneElement)
}

#[test]
fn challenge_proposal_create_only_one_item() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00013.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        // Simulate slice contains only one element.
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00013.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceMustContainMoreThanOneElement)
}

#[test]
fn challenge_proposal_create_start_with_pre_account_cell() {
    let mut template = init("propose");

    // cell_deps
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

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    // Simulate slice starting with PreAccountCell.
                    [
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
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalCellTypeError)
}

#[test]
fn challenge_proposal_create_multiple_account_cell_in_one_slice() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00005.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00005.bit",
                "next": "das00013.bit"
            },
            "witness": {
                "account": "das00005.bit"
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

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
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
                        // Simulate multiple proposal AccountCells in one slice.
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00009.bit"
                        },
                        {
                            "account_id": "das00009.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00013.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalCellTypeError)
}

#[test]
fn challenge_proposal_create_discontinued_accounts() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00010.bit"
            },
            "witness": {
                "account": "das00012.bit"
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
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            // Simulate discontinued slice, the correct next should be "das00009.bit".
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00009.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00002.bit"
                        },
                        {
                            "account_id": "das00002.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00010.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceIsDiscontinuity)
}

#[test]
fn challenge_proposal_create_invalid_order() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00010.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00002.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
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

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00002.bit"
                        },
                        // Simulate unsorted slice, the correct order should be das00012.bit -> das00005.bit -> das00002.bit
                        {
                            "account_id": "das00002.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00005.bit"
                        },
                        {
                            "account_id": "das00005.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00010.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceIsNotSorted)
}

#[test]
fn challenge_proposal_create_exist_account_1() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00002.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );
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
                "account": "das00002.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
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
                            "next": "das00002.bit"
                        },
                        // Simulate registering account that is the next of some AccountCell.
                        {
                            "account_id": "das00002.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00002.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceItemMustBeUniqueAccount)
}

#[test]
fn challenge_proposal_create_exist_account_2() {
    let mut template = init("propose");

    // cell_deps
    // CAREFUL! Slice das00009 - das00011 contains slice das00013 - das00004 , so this structure is only for testing purposes only.
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00009.bit",
                "next": "das00011.bit"
            },
            "witness": {
                "account": "das00009.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00004.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00013.bit",
                "next": "das00004.bit"
            },
            "witness": {
                "account": "das00013.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00010.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00009.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00004.bit"
                        },
                        {
                            // Simulate registering an account exists as the last next of some slice.
                            "account_id": "das00004.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00011.bit"
                        },
                    ],
                    [
                        {
                            "account_id": "das00013.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00010.bit"
                        },
                        {
                            "account_id": "das00010.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00004.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceItemMustBeUniqueAccount)
}

#[test]
fn challenge_proposal_create_exist_account_3() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00005.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00018.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00018.bit"
                        },
                        {
                            // Simulate registering an account before the next of some AccountCell.
                            "account_id": "das00018.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00005.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceIsNotSorted)
}

#[test]
fn challenge_proposal_create_exist_account_4() {
    let mut template = init("propose");

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00015.bit"
            },
            "witness": {
                "account": "das00012.bit"
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
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": "das00012.bit",
                "next": "das00015.bit"
            },
            "witness": {
                "account": "das00012.bit"
            }
        }),
    );
    push_dep_pre_account_cell(
        &mut template,
        json!({
            "witness": {
                "account": "das00013.bit",
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "created_at": TIMESTAMP - HOUR_SEC
            }
        }),
    );

    // inputs
    push_input_normal_cell(&mut template, 100_000_000_000, PROPOSER);

    // outputs
    push_output_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "slices": [
                    [
                        {
                            // Simulate use the same AccountCell in multiple slices at the same time.
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00009.bit"
                        },
                        {
                            "account_id": "das00009.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00015.bit"
                        },
                    ],
                    [
                        {
                            // Simulate use the same AccountCell in multiple slices at the same time.
                            "account_id": "das00012.bit",
                            "item_type": ProposalSliceItemType::Exist as u8,
                            "next": "das00013.bit"
                        },
                        {
                            "account_id": "das00013.bit",
                            "item_type": ProposalSliceItemType::New as u8,
                            "next": "das00015.bit"
                        },
                    ]
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ProposalSliceItemMustBeUniqueAccount)
}
