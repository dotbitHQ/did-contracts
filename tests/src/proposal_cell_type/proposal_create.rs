use super::super::util::{constants::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::*;

use super::common::*;

#[test]
fn gen_proposal_create() {
    let (mut template, height, timestamp) = init("propose");

    let slices = vec![
        vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00013.bit"),
            ("das00009.bit", ProposalSliceItemType::New, ""),
            ("das00002.bit", ProposalSliceItemType::New, ""),
        ],
        vec![
            ("das00004.bit", ProposalSliceItemType::Exist, "das00011.bit"),
            ("das00018.bit", ProposalSliceItemType::New, ""),
        ],
    ];

    let (cell_data, entity) = template.gen_proposal_cell_data(
        "0x0100000000000000000000000000000000000000",
        height,
        &slices,
    );
    template.push_proposal_cell(cell_data, Some((1, 0, entity)), 1000, Source::Output);

    gen_proposal_related_cell_at_create(&mut template, slices, timestamp);

    template.write_template("proposal_create.json");
}

test_with_template!(test_proposal_create, "proposal_create.json");

challenge_with_generator!(
    challenge_proposal_create_duplicate_account,
    Error::ProposalSliceItemMustBeUniqueAccount,
    || {
        let (mut template, height, timestamp) = init("propose");

        let slices = vec![vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00005.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ]];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0100000000000000000000000000000000000000",
            height,
            &slices,
        );
        template.push_proposal_cell(cell_data, Some((1, 0, entity)), 0, Source::Output);

        gen_proposal_related_cell_at_create(&mut template, slices, timestamp);

        template.as_json()
    }
);

test_with_generator!(test_proposal_exist_account_misunderstand, || {
    let (mut template, height, timestamp) = init("propose");

    let slices = vec![
        vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00002.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ],
        vec![
            ("das00002.bit", ProposalSliceItemType::Exist, "das00010.bit"),
            ("das00013.bit", ProposalSliceItemType::New, ""),
        ],
    ];

    let (cell_data, entity) = template.gen_proposal_cell_data(
        "0x0100000000000000000000000000000000000000",
        height,
        &slices,
    );
    template.push_proposal_cell(cell_data, Some((1, 0, entity)), 0, Source::Output);

    gen_proposal_related_cell_at_create(&mut template, slices, timestamp);

    template.as_json()
});
