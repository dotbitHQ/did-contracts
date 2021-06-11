use super::super::util::{constants::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

use super::common::*;

#[test]
fn gen_extend_proposal() {
    let (mut template, height, timestamp) = init("extend_proposal");

    // Generate previous proposal
    let slices = vec![
        vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00013.bit"),
            ("das00009.bit", ProposalSliceItemType::New, ""),
            ("das00002.bit", ProposalSliceItemType::New, ""),
        ],
        vec![
            (
                "das00010.bit", // das00006.bit
                ProposalSliceItemType::Exist,
                "das00011.bit",
            ),
            ("das00004.bit", ProposalSliceItemType::New, ""),
        ],
    ];

    let (cell_data, entity) = template.gen_proposal_cell_data(
        "0x0100000000000000000000000000000000000000",
        height - 5,
        &slices,
    );
    template.push_proposal_cell(
        cell_data,
        Some((1, template.cell_deps.len() as u32, entity)),
        1000,
        Source::CellDep,
    );

    // Generate extended proposal
    let slices = vec![
        // A slice base on previous modified AccountCell
        vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ],
        // A slice base on previous modified PreAccountCell
        vec![
            (
                "das00004.bit",
                ProposalSliceItemType::Proposed,
                "das00011.bit",
            ),
            ("das00018.bit", ProposalSliceItemType::New, ""),
            ("das00008.bit", ProposalSliceItemType::New, ""),
        ],
        // A whole new slice
        vec![
            ("das00006.bit", ProposalSliceItemType::Exist, "das00001.bit"),
            ("das00019.bit", ProposalSliceItemType::New, ""),
        ],
    ];

    let (cell_data, entity) = template.gen_proposal_cell_data(
        "0x0200000000000000000000000000000000000000",
        height,
        &slices,
    );
    template.push_proposal_cell(cell_data, Some((1, 0, entity)), 1000, Source::Output);

    gen_proposal_related_cell_at_create(&mut template, slices, timestamp);

    template.write_template("proposal_extend.json");
}

test_with_template!(test_proposal_extend, "proposal_extend.json");
