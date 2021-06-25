use super::super::util::{constants::*, template_generator::*};
use ckb_tool::ckb_types::{bytes, prelude::Pack};
use das_types::constants::*;

pub fn init(action: &str) -> (TemplateGenerator, u64, u64) {
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

pub fn gen_proposal_related_cell_at_create(
    template: &mut TemplateGenerator,
    slices: Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    timestamp: u64,
) {
    let old_registered_at = timestamp - 86400;
    let old_expired_at = timestamp + 31536000 - 86400;

    let mut dep_index = template.cell_deps.len() as u32;
    for (slice_index, slice) in slices.into_iter().enumerate() {
        println!("Generate slice {} ...", slice_index);

        for (item_index, (account, item_type, next_account)) in slice.iter().enumerate() {
            println!(
                "  Generate item {}: {}",
                item_index,
                bytes::Bytes::from(account_to_id_bytes(account)).pack()
            );

            if *item_type == ProposalSliceItemType::Exist {
                let (cell_data, entity) = template.gen_account_cell_data(
                    account,
                    next_account,
                    old_registered_at,
                    old_expired_at,
                    0,
                    0,
                    0,
                    None,
                );
                template.push_account_cell(
                    "0x0000000000000000000000000000000000001111",
                    "0x0000000000000000000000000000000000001111",
                    cell_data,
                    Some((1, dep_index, entity)),
                    1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
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
                    476_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
                    Source::CellDep,
                );
            }

            dep_index += 1;
        }
    }
}

macro_rules! gen_account_cells {
    ($template:expr, $account:expr, $next:expr, $updated_next:expr, $registered_at:expr, $expired_at:expr, $input_index:expr, $output_index:expr) => {{
        // Generate AccountCell in inputs
        let (cell_data, old_entity) = $template.gen_account_cell_data(
            $account,
            $next,
            $registered_at,
            $expired_at,
            0,
            0,
            0,
            None,
        );
        $template.push_account_cell(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        // Generate AccountCell in outputs
        let (cell_data, new_entity) = $template.gen_account_cell_data(
            $account,
            $updated_next,
            $registered_at,
            $expired_at,
            0,
            0,
            0,
            None,
        );
        $template.push_account_cell(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        // Generate witness of AccountCell.
        $template.push_witness(
            DataType::AccountCellData,
            Some((1, $output_index, new_entity)),
            Some((1, $input_index, old_entity)),
            None,
        );
    }};
}

macro_rules! gen_account_cells_edit_capacity {
    ($template:expr, $account:expr, $next:expr, $updated_next:expr, $registered_at:expr, $expired_at:expr, $input_index:expr, $output_index:expr, $input_capacity:expr, $output_capacity:expr) => {{
        // Generate AccountCell in inputs
        let (cell_data, old_entity) = $template.gen_account_cell_data(
            $account,
            $next,
            $registered_at,
            $expired_at,
            0,
            0,
            0,
            None,
        );
        $template.push_account_cell(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data,
            None,
            $input_capacity, // The capacity is edited
            Source::Input,
        );

        // Generate AccountCell in outputs
        let (cell_data, new_entity) = $template.gen_account_cell_data(
            $account,
            $updated_next,
            $registered_at,
            $expired_at,
            0,
            0,
            0,
            None,
        );
        $template.push_account_cell(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data,
            None,
            $output_capacity, // The capacity is edited
            Source::Output,
        );

        // Generate witness of AccountCell.
        $template.push_witness(
            DataType::AccountCellData,
            Some((1, $output_index, new_entity)),
            Some((1, $input_index, old_entity)),
            None,
        );
    }};
}

macro_rules! gen_account_and_pre_account_cells {
    ($template:expr, $account:expr, $next:expr, $quote:expr, $invited_discount:expr, $created_at:expr, $registered_at:expr, $expired_at:expr, $input_index:expr, $output_index:expr) => {{
        // Generate PreAccountCell in inputs.
        let (cell_data, entity) = $template.gen_pre_account_cell_data(
            $account,
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001100",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            $quote,
            $invited_discount,
            $created_at,
        );
        $template.push_pre_account_cell(
            cell_data,
            Some((1, $input_index, entity)),
            476_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        // Generate new AccountCell in outputs.
        let (cell_data, entity) = $template.gen_account_cell_data(
            $account,
            $next,
            $registered_at,
            $expired_at,
            0,
            0,
            0,
            None,
        );
        $template.push_account_cell(
            "0x0000000000000000000000000000000000001100",
            "0x0000000000000000000000000000000000001100",
            cell_data,
            Some((1, $output_index, entity)),
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );
    }};
}

macro_rules! gen_account_and_pre_account_cells_edit_capacity {
    ($template:expr, $account:expr, $next:expr, $quote:expr, $invited_discount:expr, $created_at:expr, $registered_at:expr, $expired_at:expr, $input_index:expr, $output_index:expr, $input_capacity:expr, $output_capacity:expr) => {{
        // Generate PreAccountCell in inputs.
        let (cell_data, entity) = $template.gen_pre_account_cell_data(
            $account,
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001100",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            $quote,
            $invited_discount,
            $created_at,
        );
        $template.push_pre_account_cell(
            cell_data,
            Some((1, $input_index, entity)),
            $input_capacity,
            Source::Input,
        );

        // Generate new AccountCell in outputs.
        let (cell_data, entity) = $template.gen_account_cell_data(
            $account,
            $next,
            $registered_at,
            $expired_at,
            0,
            0,
            0,
            None,
        );
        $template.push_account_cell(
            "0x0000000000000000000000000000000000001100",
            "0x0000000000000000000000000000000000001100",
            cell_data,
            Some((1, $output_index, entity)),
            $output_capacity,
            Source::Output,
        );
    }};
}
