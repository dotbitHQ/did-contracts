use super::common::*;
use crate::util::{
    accounts::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator, template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn before_each(height: u64) -> TemplateGenerator {
    let mut template = init("recycle_proposal");

    push_input_proposal_cell(
        &mut template,
        json!({
            "witness": {
                "created_at_height": height,
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

    template
}

#[test]
fn test_proposal_recycle() {
    let mut template = before_each(HEIGHT - 6);

    // outputs
    push_output_normal_cell(&mut template, 20_000_000_000, COMMON_PROPOSER);

    test_tx(template.as_json());
}

#[test]
fn challenge_proposal_recycle_too_early() {
    let mut template = before_each(HEIGHT - 5);

    // outputs
    push_output_normal_cell(&mut template, 20_000_000_000, COMMON_PROPOSER);

    challenge_tx(template.as_json(), Error::ProposalRecycleNeedWaitLonger);
}

#[test]
fn challenge_proposal_recycle_refund_capacity() {
    let mut template = before_each(HEIGHT - 6);

    // outputs
    push_output_normal_cell(&mut template, 20_000_000_000 - 10000 - 1, COMMON_PROPOSER);

    challenge_tx(template.as_json(), Error::ProposalConfirmRefundError);
}

#[test]
fn challenge_proposal_recycle_refund_owner() {
    let mut template = before_each(HEIGHT - 6);

    // outputs
    push_output_normal_cell(
        &mut template,
        20_000_000_000,
        "0x0000000000000000000000000000000000002233",
    );

    challenge_tx(template.as_json(), Error::ProposalConfirmRefundError);
}
