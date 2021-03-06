use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes, prelude::Pack};
use das_types::constants::*;

fn gen_cell_deps(template: &mut TemplateGenerator, height: u64, timestamp: u64) {
    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);
    template.push_height_cell(1, height, 200_000_000_000, Source::CellDep);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        true,
        100_000_000_000,
        Source::CellDep,
    );
}

fn gen_proposal_related_cell_at_create(
    template: &mut TemplateGenerator,
    slices: Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    timestamp: u64,
    start_from: u32,
) {
    let old_registered_at = timestamp - 86400;
    let old_expired_at = timestamp + 31536000 - 86400;

    let mut dep_index = start_from;
    for (slice_index, slice) in slices.into_iter().enumerate() {
        println!("Generate slice {} ...", slice_index);

        for (item_index, (account, item_type, next)) in slice.iter().enumerate() {
            println!(
                "  Generate item {}: {}",
                item_index,
                bytes::Bytes::from(account_to_id_bytes(account)).pack()
            );

            let splited_account = account[..&account.len() - 4]
                .split("")
                .filter(|item| !item.is_empty())
                .collect::<Vec<&str>>();
            let account_chars = gen_account_chars(splited_account);

            if *item_type == ProposalSliceItemType::Exist {
                let origin_next = bytes::Bytes::from(account_to_id_bytes(next));
                let (cell_data, entity) = template.gen_account_cell_data(
                    &account_chars,
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    origin_next.clone(),
                    old_registered_at,
                    old_expired_at,
                );
                template.push_account_cell(
                    cell_data,
                    Some((1, dep_index, entity)),
                    194,
                    Source::CellDep,
                );
            } else {
                let (cell_data, entity) = template.gen_pre_account_cell_data(
                    &account_chars,
                    "0x0000000000000000000000000000000000002222",
                    "0x000000000000000000000000000000000000FFFF",
                    "inviter_01.bit",
                    "channel_01.bit",
                    1000,
                    timestamp - 60,
                );
                template.push_pre_account_cell(
                    cell_data,
                    Some((1, dep_index, entity)),
                    5308,
                    Source::CellDep,
                );
            }

            dep_index += 1;
        }
    }
}

// #[test]
fn gen_proposal_create_test_data() {
    println!("====== Print propose transaction data ======");

    let mut template = TemplateGenerator::new("propose", None);
    let height = 1000u64;
    let timestamp = 1611200090u64;

    gen_cell_deps(&mut template, height, timestamp);

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
        "proposer_01.bit",
        height,
        &slices,
    );
    template.push_proposal_cell(cell_data, Some((1, 0, entity)), 1000, Source::Output);

    gen_proposal_related_cell_at_create(&mut template, slices, timestamp, 4);

    template.pretty_print();
}

// #[test]
fn test_proposal_create() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "proposal_create.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_propose: {} cycles", cycles);
}

// #[test]
fn gen_extend_proposal_test_data() {
    println!("====== Print extend proposal transaction data ======");

    let mut template = TemplateGenerator::new("extend_proposal", None);
    let height = 1000u64;
    let timestamp = 1611200090u64;

    gen_cell_deps(&mut template, height, timestamp);

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
        "proposer_01.bit",
        height - 5,
        &slices,
    );
    template.push_proposal_cell(cell_data, Some((1, 5, entity)), 1000, Source::CellDep);

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
        "proposer_02.bit",
        height,
        &slices,
    );
    template.push_proposal_cell(cell_data, Some((1, 0, entity)), 1000, Source::Output);

    gen_proposal_related_cell_at_create(&mut template, slices, timestamp, 4);

    template.pretty_print();
}

// #[test]
fn test_extend_proposal() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "proposal_extend.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_extend_proposal: {} cycles", cycles);
}

fn gen_proposal_related_cell_at_confirm(
    template: &mut TemplateGenerator,
    slices: Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    timestamp: u64,
) {
    let old_registered_at = timestamp - 86400;
    let old_expired_at = timestamp + 31536000 - 86400;
    let new_registered_at = timestamp;
    let new_expired_at = timestamp + 31536000;

    let mut input_index = 1;
    let mut output_index = 0;
    let mut accounts_to_gen_ref_cells = Vec::new();
    for (slice_index, slice) in slices.into_iter().enumerate() {
        println!("Generate slice {} ...", slice_index);

        let mut next_of_first_item = bytes::Bytes::default();
        for (item_index, (account, item_type, next)) in slice.iter().enumerate() {
            println!(
                "  Generate item {}: {}",
                item_index,
                bytes::Bytes::from(account_to_id_bytes(account)).pack()
            );

            let splited_account = account[..&account.len() - 4]
                .split("")
                .filter(|item| !item.is_empty())
                .collect::<Vec<&str>>();
            let account_chars = gen_account_chars(splited_account);

            if *item_type == ProposalSliceItemType::Exist
                || *item_type == ProposalSliceItemType::Proposed
            {
                // Generate old AccountCell in inputs.
                let origin_next = bytes::Bytes::from(account_to_id_bytes(next));
                println!("    📥 next_of_first_item: {}", origin_next.pack());
                next_of_first_item = origin_next.clone();
                let (cell_data, new_entity) = template.gen_account_cell_data(
                    &account_chars,
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    origin_next.clone(),
                    old_registered_at,
                    old_expired_at,
                );
                template.push_account_cell(cell_data, None, 15_800_000_000, Source::Input);

                // Generate new AccountCell in outputs.
                let (account, _, _) = slice.get(item_index + 1).unwrap();
                let updated_next = bytes::Bytes::from(account_to_id_bytes(account));
                let (cell_data, old_entity) = template.gen_account_cell_data(
                    &account_chars,
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    updated_next.clone(),
                    old_registered_at,
                    old_expired_at,
                );
                template.push_account_cell(cell_data, None, 15_800_000_000, Source::Output);

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
                    &account_chars,
                    "0x0000000000000000000000000000000000002222",
                    "0x000000000000000000000000000000000000FFFF",
                    "inviter_01.bit",
                    "channel_01.bit",
                    1000,
                    timestamp - 60,
                );
                template.push_pre_account_cell(
                    cell_data,
                    Some((1, input_index, entity)),
                    524_200_000_000,
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
                    &account_chars,
                    "0x0000000000000000000000000000000000002222",
                    "0x0000000000000000000000000000000000002222",
                    updated_next.clone(),
                    new_registered_at,
                    new_expired_at,
                );
                template.push_account_cell(
                    cell_data,
                    Some((1, output_index, entity)),
                    15_800_000_000,
                    Source::Output,
                );

                println!(
                    "    Item {} next: None -> {}",
                    item_index,
                    updated_next.pack()
                );

                // Generate new RefCell in outputs.
                accounts_to_gen_ref_cells.push(*account)
            }

            input_index += 1;
            output_index += 1;
        }
    }

    for account in accounts_to_gen_ref_cells {
        template.push_ref_cell(
            "0x0000000000000000000000000000000000000011",
            account,
            9_400_000_000,
            Source::Output,
        );
    }
}

// #[test]
fn gen_confirm_proposal_test_data() {
    println!("====== Print confirm proposal transaction data ======");

    let mut template = TemplateGenerator::new("confirm_proposal", None);
    let height = 1000u64;
    let timestamp = 1611200090u64;

    gen_cell_deps(&mut template, height, timestamp);

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
        "0x0100000000000000000000000000000000000000",
        "proposer_01.bit",
        height,
        &slices,
    );
    template.push_proposal_cell(
        cell_data,
        Some((1, 0, entity)),
        100_000_000_000,
        Source::Input,
    );

    gen_proposal_related_cell_at_confirm(&mut template, slices, timestamp);

    template.push_wallet_cell("inviter_01.bit", 9_400_000_000, Source::Input);
    template.push_wallet_cell("channel_01.bit", 9_400_000_000, Source::Input);
    template.push_wallet_cell("das.bit", 9_400_000_000, Source::Input);
    template.push_wallet_cell("inviter_01.bit", 209_400_000_000, Source::Output);
    template.push_wallet_cell("channel_01.bit", 209_400_000_000, Source::Output);
    template.push_wallet_cell("das.bit", 1609_400_000_000, Source::Output);

    template.pretty_print();
}

// #[test]
fn test_proposal_confirm() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "proposal_confirm.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_confirm_proposal: {} cycles", cycles);
}

// #[test]
fn gen_proposal_recycle_test_data() {
    println!("====== Print recycle proposal transaction data ======");

    let mut template = TemplateGenerator::new("recycle_proposal", None);
    let height = 1000u64;

    template.push_height_cell(1, height, 200_000_000_000, Source::CellDep);
    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        true,
        100_000_000_000,
        Source::CellDep,
    );

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
        "0x0100000000000000000000000000000000000000",
        "proposer_01.bit",
        height - 10,
        &slices,
    );
    template.push_proposal_cell(
        cell_data,
        Some((1, 0, entity)),
        100_000_000_000,
        Source::Input,
    );

    template.pretty_print();
}

#[test]
fn test_proposal_recycle() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "proposal_recycle.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_recycle_proposal: {} cycles", cycles);
}
