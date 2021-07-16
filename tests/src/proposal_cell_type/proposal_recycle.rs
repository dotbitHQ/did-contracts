use super::super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::*;

fn init_recycle() -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new("recycle_proposal", None);
    let height = 1000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("proposal-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Height, height);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProposal, true, 0, Source::CellDep);

    (template, height)
}

#[test]
fn gen_proposal_recycle() {
    let (mut template, height) = init_recycle();

    let slices = vec![
        // A slice base on previous modified AccountCell
        vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ],
    ];

    // inputs
    let (cell_data, entity) = template.gen_proposal_cell_data(
        "0x0000000000000000000000000000000000002233",
        height - 10,
        &slices,
    );
    template.push_proposal_cell(
        cell_data,
        Some((1, 0, entity)),
        100_000_000_000,
        Source::Input,
    );

    // outputs
    template.push_signall_cell(
        "0x0000000000000000000000000000000000002233",
        100_000_000_000 - 10000,
        Source::Output,
    );

    template.write_template("proposal_recycle.json");
}

test_with_template!(test_proposal_recycle, "proposal_recycle.json");

challenge_with_generator!(
    chanllenge_proposal_recycle_too_early,
    Error::ProposalRecycleNeedWaitLonger,
    || {
        let (mut template, height) = init_recycle();

        let slices = vec![
            // A slice base on previous modified AccountCell
            vec![
                ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
                ("das00005.bit", ProposalSliceItemType::New, ""),
            ],
        ];

        // inputs
        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height - 5,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        // outputs
        template.push_signall_cell(
            "0x0000000000000000000000000000000000002233",
            100_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    chanllenge_proposal_recycle_refund_error,
    Error::ProposalConfirmRefundError,
    || {
        let (mut template, height) = init_recycle();

        // inputs
        let slices = vec![
            // A slice base on previous modified AccountCell
            vec![
                ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
                ("das00005.bit", ProposalSliceItemType::New, ""),
            ],
        ];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height - 10,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        // outputs
        template.push_signall_cell(
            "0x0000000000000000000000000000000000002233",
            100_000_000_000 - 10001,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    chanllenge_proposal_recycle_no_refund,
    Error::ProposalConfirmRefundError,
    || {
        let (mut template, height) = init_recycle();

        // inputs
        let slices = vec![
            // A slice base on previous modified AccountCell
            vec![
                ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
                ("das00005.bit", ProposalSliceItemType::New, ""),
            ],
        ];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height - 10,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        template.as_json()
    }
);
