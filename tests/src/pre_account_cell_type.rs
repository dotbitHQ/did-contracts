use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::*;

fn gen_cell_deps(template: &mut TemplateGenerator, height: u64, timestamp: u64) {
    template.push_contract_cell("always_success", true);
    template.push_contract_cell("config-cell-type", false);
    template.push_contract_cell("apply-register-cell-type", false);
    template.push_contract_cell("pre-account-cell-type", false);

    template.push_height_cell(1, height, 200_000_000_000, Source::CellDep);
    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_quote_cell(1000, 500_000_000_000, Source::CellDep);

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
    template.push_config_cell(
        ConfigID::ConfigCellBloomFilter,
        true,
        100_000_000_000,
        Source::CellDep,
    );
}

fn init(account: &str) -> (TemplateGenerator, &str, u64) {
    let mut template = TemplateGenerator::new("pre_register", None);
    let timestamp = 1611200060u64;
    let height = 1000u64;

    gen_cell_deps(&mut template, height, timestamp);

    template.push_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        account,
        height - 4,
        0,
        Source::Input,
    );

    (template, account, timestamp)
}

#[test]
fn gen_pre_register() {
    let (mut template, account, timestamp) = init("das00001.bit");

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x0000000000000000000000000000000000002222",
        "0x000000000000000000000000000000000000FFFF",
        "inviter_01.bit",
        "channel_01.bit",
        1000,
        500,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        535_600_000_000,
        Source::Output,
    );

    template.pretty_print();
}

test_with_template!(test_pre_register, "pre_register.json");

challenge_with_generator!(
    challenge_pre_register_reserved_account,
    Error::AccountIsReserved,
    || {
        let (mut template, account, timestamp) = init("microsoft.bit");

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "inviter_01.bit",
            "channel_01.bit",
            1000,
            500,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            535_600_000_000,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_account_length,
    Error::AccountStillCanNotBeRegister,
    || {
        let (mut template, account, timestamp) = init("a.bit");

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "inviter_01.bit",
            "channel_01.bit",
            1000,
            500,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            1174900000000,
            Source::Output,
        );

        template.as_json()
    }
);
