use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes, prelude::Pack};
use das_core::error::Error;
use das_types::constants::*;

fn init(action: &str) -> (TemplateGenerator, u64, u64) {
    let mut template = TemplateGenerator::new(action, None);
    let height = 1000u64;
    let timestamp = 1611200090u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("proposal-cell-type", false);

    template.push_height_cell(1, height, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProposal, true, 0, Source::CellDep);

    (template, height, timestamp)
}

fn gen_proposal_related_cell_at_create(
    template: &mut TemplateGenerator,
    slices: Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    timestamp: u64,
) {
    let old_registered_at = timestamp - 86400;
    let old_expired_at = timestamp + 31536000 - 86400;

    let mut dep_index = template.cell_deps.len() as u32;
    for (slice_index, slice) in slices.into_iter().enumerate() {
        println!("Generate slice {} ...", slice_index);

        for (item_index, (account, item_type, next)) in slice.iter().enumerate() {
            println!(
                "  Generate item {}: {}",
                item_index,
                bytes::Bytes::from(account_to_id_bytes(account)).pack()
            );

            if *item_type == ProposalSliceItemType::Exist {
                let origin_next = bytes::Bytes::from(account_to_id_bytes(next));
                let (cell_data, entity) = template.gen_account_cell_data(
                    account,
                    origin_next.clone(),
                    old_registered_at,
                    old_expired_at,
                    None,
                );
                template.push_account_cell(
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    cell_data,
                    Some((1, dep_index, entity)),
                    15_800_000_000,
                    Source::CellDep,
                );
            } else {
                let (cell_data, entity) = template.gen_pre_account_cell_data(
                    account,
                    "0x000000000000000000000000000000000000FFFF",
                    "0x0000000000000000000000000000000000001100",
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000002222",
                    1000,
                    500,
                    timestamp - 60,
                );
                template.push_pre_account_cell(
                    cell_data,
                    Some((1, dep_index, entity)),
                    535_600_000_000,
                    Source::CellDep,
                );
            }

            dep_index += 1;
        }
    }
}

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

    template.pretty_print();
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

    template.pretty_print();
}

test_with_template!(test_proposal_extend, "proposal_extend.json");

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

        let mut next_of_first_item = bytes::Bytes::default();
        for (item_index, (account, item_type, next)) in slice.iter().enumerate() {
            println!(
                "  Generate item {}: {}",
                item_index,
                bytes::Bytes::from(account_to_id_bytes(account)).pack()
            );

            if *item_type == ProposalSliceItemType::Exist
                || *item_type == ProposalSliceItemType::Proposed
            {
                // Generate old AccountCell in inputs.
                let origin_next = bytes::Bytes::from(account_to_id_bytes(next));
                println!("    📥 next_of_first_item: {}", origin_next.pack());
                next_of_first_item = origin_next.clone();
                let (cell_data, old_entity) = template.gen_account_cell_data(
                    account,
                    origin_next.clone(),
                    old_registered_at,
                    old_expired_at,
                    None,
                );
                template.push_account_cell(
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    cell_data,
                    None,
                    20_000_000_000,
                    Source::Input,
                );

                // Generate new AccountCell in outputs.
                let (next_account, _, _) = slice.get(item_index + 1).unwrap();
                let updated_next = bytes::Bytes::from(account_to_id_bytes(next_account));
                let (cell_data, new_entity) = template.gen_account_cell_data(
                    account,
                    updated_next.clone(),
                    old_registered_at,
                    old_expired_at,
                    None,
                );
                template.push_account_cell(
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    cell_data,
                    None,
                    20_000_000_000,
                    Source::Output,
                );

                println!(
                    "    Item {} next: {} -> {}",
                    item_index,
                    origin_next.pack(),
                    updated_next.pack()
                );

                // Generate witness of AccountCell.
                template.push_witness(
                    DataType::AccountCellData,
                    Some((1, output_index, new_entity)),
                    Some((1, input_index, old_entity)),
                    None,
                );
            } else {
                // Generate old PreAccountCell in inputs.
                let (cell_data, entity) = template.gen_pre_account_cell_data(
                    account,
                    "0x000000000000000000000000000000000000FFFF",
                    "0x0000000000000000000000000000000000001100",
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000002222",
                    1000,
                    500,
                    timestamp - 60,
                );
                template.push_pre_account_cell(
                    cell_data,
                    Some((1, input_index, entity)),
                    496_200_000_000,
                    Source::Input,
                );

                // Generate new AccountCell in outputs.
                let updated_next = if item_index != slice.len() - 1 {
                    let (account, _, _) = slice.get(item_index + 1).unwrap();
                    bytes::Bytes::from(account_to_id_bytes(account))
                } else {
                    println!("    📤 next_of_first_item");
                    next_of_first_item.clone()
                };
                let (cell_data, entity) = template.gen_account_cell_data(
                    account,
                    updated_next.clone(),
                    new_registered_at,
                    new_expired_at,
                    None,
                );
                template.push_account_cell(
                    "0x0000000000000000000000000000000000001100",
                    "0x0000000000000000000000000000000000001100",
                    cell_data,
                    Some((1, output_index, entity)),
                    20_000_000_000,
                    Source::Output,
                );

                println!(
                    "    Item {} next: None -> {}",
                    item_index,
                    updated_next.pack()
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
fn gen_confirm_proposal() {
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

    template.pretty_print();
}

test_with_template!(test_proposal_confirm, "proposal_confirm.json");

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
    chanllenge_proposal_confirm_income_capacity_mismatch,
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
            20_000_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

fn init_recycle() -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new("recycle_proposal", None);
    let height = 1000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("proposal-cell-type", false);

    template.push_height_cell(1, height, 0, Source::CellDep);
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

    template.push_signall_cell(
        "0x0000000000000000000000000000000000002233",
        100_000_000_000,
        Source::Output,
    );

    template.pretty_print();
}

test_with_template!(test_proposal_recycle, "proposal_recycle.json");

challenge_with_generator!(
    chanllenge_proposal_recycle_no_refund,
    Error::ProposalRecycleCanNotFoundRefundCell,
    || {
        let (mut template, height) = init_recycle();

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
