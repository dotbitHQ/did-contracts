use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::*;

fn init(account: &str) -> (TemplateGenerator, &str, u64) {
    let mut template = TemplateGenerator::new("pre_register", None);
    let timestamp = 1611200060u64;
    let height = 1000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("apply-register-cell-type", false);
    template.push_contract_cell("pre-account-cell-type", false);

    template.push_height_cell(1, height, 200_000_000_000, Source::CellDep);
    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_quote_cell(1000, 500_000_000_000, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEmoji, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::CellDep);
    template.push_config_cell(
        DataType::ConfigCellPreservedAccount00,
        true,
        0,
        Source::CellDep,
    );

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
    let (mut template, account, timestamp) = init("✨das🎉001.bit");

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x000000000000000000000000000000000000FFFF",
        "0x0000000000000000000000000000000000001100",
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        1000,
        500,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        476_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.write_template("pre_register.json");
}

test_with_template!(test_pre_register, "pre_register.json");

test_with_generator!(test_pre_register_char_set, || {
    let (mut template, account, timestamp) = init("一二三四0001.bit");
    template.push_config_cell(DataType::ConfigCellCharSetZhHans, true, 0, Source::CellDep);

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x000000000000000000000000000000000000FFFF",
        "0x0000000000000000000000000000000000001100",
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        1000,
        500,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        476_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_pre_register_invalid_char,
    Error::PreRegisterAccountCharIsInvalid,
    || {
        // ⚠️ Need delete the emoji from char_set_emoji.txt first, otherwise the test can not pass.
        let (mut template, account, timestamp) = init("✨das🎱001.bit");

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            1000,
            500,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            476_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_preserved_account,
    Error::AccountIsPreserved,
    || {
        let (mut template, account, timestamp) = init("microsoft.bit");

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            1000,
            500,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            476_300_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_account_can_not_register,
    Error::AccountStillCanNotBeRegister,
    || {
        let (mut template, account, timestamp) = init("a.bit");

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            1000,
            500,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            1_140_500_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_pre_register_account_length,
    Error::PreRegisterAccountIsTooLong,
    || {
        let (mut template, account, timestamp) = init("123456789012345678901.bit");

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            1000,
            500,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            500_000_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.as_json()
    }
);
