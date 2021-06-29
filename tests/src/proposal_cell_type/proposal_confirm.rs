use super::super::util;
use super::super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::{constants::*, packed::*};

fn gen_proposal_related_cell_at_confirm(
    template: &mut TemplateGenerator,
    slices: Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    timestamp: u64,
) -> (u32, u32) {
    let old_registered_at = timestamp - 86400;
    let old_expired_at = timestamp + 31536000 - 86400;
    let new_registered_at = timestamp;
    let new_expired_at = timestamp + 31536000;

    let mut input_index: u32 = 1;
    let mut output_index: u32 = 0;
    for (slice_index, slice) in slices.into_iter().enumerate() {
        println!("Generate slice {} ...", slice_index);

        let mut next_of_first_item = "";
        for (item_index, (account, item_type, next_account)) in slice.iter().enumerate() {
            if *item_type == ProposalSliceItemType::Exist
                || *item_type == ProposalSliceItemType::Proposed
            {
                // Generate old AccountCell in inputs.
                println!(
                    "    📥 next_of_first_item: {}",
                    util::account_to_id_hex(next_account)
                );
                next_of_first_item = next_account;
                let (updated_next_account, _, _) = slice.get(item_index + 1).unwrap();

                gen_account_cells!(
                    template,
                    account,
                    next_account,
                    updated_next_account,
                    old_registered_at,
                    old_expired_at,
                    input_index,
                    output_index
                );

                println!(
                    "    Item {}: {} -> {}",
                    item_index,
                    util::account_to_id_hex(next_account),
                    util::account_to_id_hex(updated_next_account)
                );
            } else {
                let next_account = if item_index != slice.len() - 1 {
                    let (account, _, _) = slice.get(item_index + 1).unwrap();
                    account
                } else {
                    println!("    📤 next_of_first_item");
                    next_of_first_item
                };

                gen_account_and_pre_account_cells!(
                    template,
                    account,
                    next_account,
                    1000,
                    500,
                    timestamp - 60,
                    new_registered_at,
                    new_expired_at,
                    input_index,
                    output_index
                );

                println!(
                    "    Item {} next: None -> {}",
                    item_index,
                    util::account_to_id_hex(next_account)
                );
            }

            input_index += 1;
            output_index += 1;
        }
    }

    (input_index, output_index)
}

fn init_confirm(action: &str) -> (TemplateGenerator, u64, u64) {
    let height = 1000u64;
    let timestamp = 1611200090u64;
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("proposal-cell-type", false);
    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("pre-account-cell-type", false);
    template.push_contract_cell("income-cell-type", false);

    template.push_time_cell(1, timestamp, 0, Source::CellDep);
    template.push_height_cell(1, height, 0, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);

    (template, height, timestamp)
}

#[test]
fn gen_proposal_confirm() {
    let (mut template, height, timestamp) = init_confirm("confirm_proposal");

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
        "0x0000000000000000000000000000000000002233",
        height,
        &slices,
    );
    template.push_proposal_cell(
        cell_data,
        Some((1, 0, entity)),
        100_000_000_000,
        Source::Input,
    );

    let (input_index, output_index) =
        gen_proposal_related_cell_at_confirm(&mut template, slices, timestamp);

    let income_records = vec![IncomeRecordParam {
        belong_to: "0x0000000000000000000000000000000000000000",
        capacity: 20_000_000_000,
    }];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(
        cell_data,
        Some((1, input_index, entity)),
        20_000_000_000,
        Source::Input,
    );

    let income_records = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000",
            capacity: 20_000_000_000,
        },
        // Profit to inviter
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000001111",
            capacity: 152_000_000_000,
        },
        // Profit to channel
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000002222",
            capacity: 152_000_000_000,
        },
        // Profit to proposer
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000002233",
            capacity: 76_000_000_000,
        },
        // Profit to DAS
        IncomeRecordParam {
            belong_to: "0x0300000000000000000000000000000000000000",
            capacity: 1_520_000_000_000,
        },
    ];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(
        cell_data,
        Some((1, output_index, entity)),
        1_920_000_000_000,
        Source::Output,
    );

    template.push_signall_cell(
        "0x0000000000000000000000000000000000002233",
        100_000_000_000,
        Source::Output,
    );

    template.write_template("proposal_confirm.json");
}

test_with_template!(test_proposal_confirm, "proposal_confirm.json");

macro_rules! gen_income_cell {
    ($template:expr, $output_index:expr) => {{
        let income_records = vec![
            // Profit to inviter
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000001111",
                capacity: 38_000_000_000,
            },
            // Profit to channel
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000002222",
                capacity: 38_000_000_000,
            },
            // Profit to proposer
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000002233",
                capacity: 19_000_000_000,
            },
            // Profit to DAS
            IncomeRecordParam {
                belong_to: "0x0300000000000000000000000000000000000000",
                capacity: 380_000_000_000,
            },
        ];
        let (cell_data, entity) = $template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        $template.push_income_cell(
            cell_data,
            Some((1, $output_index, entity)),
            475_000_000_000,
            Source::Output,
        );
    }};
}

challenge_with_generator!(
    chanllenge_proposal_confirm_no_refund,
    Error::ProposalConfirmRefundError,
    || {
        let (mut template, height, timestamp) = init_confirm("confirm_proposal");

        let slices = vec![vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ]];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        let (_, output_index) =
            gen_proposal_related_cell_at_confirm(&mut template, slices, timestamp);

        gen_income_cell!(template, output_index);

        template.as_json()
    }
);

challenge_with_generator!(
    chanllenge_proposal_confirm_income_record_belong_to_mismatch,
    Error::ProposalConfirmIncomeError,
    || {
        let (mut template, height, timestamp) = init_confirm("confirm_proposal");

        let slices = vec![vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ]];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        let (_, output_index) =
            gen_proposal_related_cell_at_confirm(&mut template, slices, timestamp);

        let income_records = vec![
            // Profit to inviter
            IncomeRecordParam {
                belong_to: "0x000000000000000000000000000000000000FFFF",
                capacity: 38_000_000_000,
            },
            // Profit to channel
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000002222",
                capacity: 38_000_000_000,
            },
            // Profit to proposer
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000002233",
                capacity: 19_000_000_000,
            },
            // Profit to DAS
            IncomeRecordParam {
                belong_to: "0x0300000000000000000000000000000000000000",
                capacity: 380_000_000_000,
            },
        ];
        let (cell_data, entity) = template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(
            cell_data,
            Some((1, output_index, entity)),
            475_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    chanllenge_proposal_confirm_income_record_capacity_mismatch,
    Error::ProposalConfirmIncomeError,
    || {
        let (mut template, height, timestamp) = init_confirm("confirm_proposal");

        let slices = vec![vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ]];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        let (_, output_index) =
            gen_proposal_related_cell_at_confirm(&mut template, slices, timestamp);

        let income_records = vec![
            // Profit to inviter
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000001111",
                capacity: 99_000_000_000,
            },
            // Profit to channel
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000002222",
                capacity: 38_000_000_000,
            },
            // Profit to proposer
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000002233",
                capacity: 19_000_000_000,
            },
            // Profit to DAS
            IncomeRecordParam {
                belong_to: "0x0300000000000000000000000000000000000000",
                capacity: 380_000_000_000,
            },
        ];
        let (cell_data, entity) = template
            .gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(
            cell_data,
            Some((1, output_index, entity)),
            475_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    chanllenge_proposal_confirm_account_cell_capacity_mismatch,
    Error::CellCapacityMustConsistent,
    || {
        let (mut template, height, timestamp) = init_confirm("confirm_proposal");

        let slices = vec![vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ]];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        let old_registered_at = timestamp - 86400;
        let old_expired_at = timestamp + 31536000 - 86400;
        let new_registered_at = timestamp;
        let new_expired_at = timestamp + 31536000;

        gen_account_cells_edit_capacity!(
            template,
            "das00012.bit",
            "das00009.bit",
            "das00005.bit",
            old_registered_at,
            old_expired_at,
            1,
            0,
            20_000_000_000,
            19_900_000_000
        );
        gen_account_and_pre_account_cells!(
            template,
            "das00005.bit",
            "das00009.bit",
            1000,
            500,
            timestamp - 60,
            new_registered_at,
            new_expired_at,
            2,
            1
        );

        gen_income_cell!(template, 2);

        template.as_json()
    }
);

challenge_with_generator!(
    chanllenge_proposal_confirm_new_account_cell_capacity_mismatch,
    Error::ProposalConfirmNewAccountCellCapacityError,
    || {
        let (mut template, height, timestamp) = init_confirm("confirm_proposal");

        let slices = vec![vec![
            ("das00012.bit", ProposalSliceItemType::Exist, "das00009.bit"),
            ("das00005.bit", ProposalSliceItemType::New, ""),
        ]];

        let (cell_data, entity) = template.gen_proposal_cell_data(
            "0x0000000000000000000000000000000000002233",
            height,
            &slices,
        );
        template.push_proposal_cell(
            cell_data,
            Some((1, 0, entity)),
            100_000_000_000,
            Source::Input,
        );

        let old_registered_at = timestamp - 86400;
        let old_expired_at = timestamp + 31536000 - 86400;
        let new_registered_at = timestamp;
        let new_expired_at = timestamp + 31536000;

        gen_account_cells!(
            template,
            "das00012.bit",
            "das00009.bit",
            "das00005.bit",
            old_registered_at,
            old_expired_at,
            1,
            0
        );
        gen_account_and_pre_account_cells_edit_capacity!(
            template,
            "das00005.bit",
            "das00009.bit",
            1000,
            500,
            timestamp - 60,
            new_registered_at,
            new_expired_at,
            2,
            1,
            496_200_000_000,
            19_900_000_000
        );

        gen_income_cell!(template, 2);

        template.as_json()
    }
);
