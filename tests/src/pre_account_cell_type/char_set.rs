use super::common::init;
use crate::util::{constants::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::constants::*;

test_with_generator!(test_pre_register_char_set, || {
    let (mut template, account, timestamp) = init("✨咐桑糯0001.bit");
    template.push_config_cell_derived_by_account("✨咐桑糯0001", true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetZhHans, true, 0, Source::CellDep);

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x000000000000000000000000000000000000FFFF",
        "0x0000000000000000000000000000000000001100",
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        CKB_QUOTE,
        INVITED_DISCOUNT,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        util::gen_register_fee(8, true),
        Source::Output,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_pre_register_invalid_char,
    Error::PreRegisterAccountCharIsInvalid,
    || {
        // ⚠️ Need to delete the emoji from char_set_emoji.txt first, otherwise the test can not pass.
        let (mut template, account, timestamp) = init("✨das🎱001.bit");
        template.push_config_cell_derived_by_account("✨das🎱001", true, 0, Source::CellDep);

        let (cell_data, entity) = template.gen_pre_account_cell_data(
            account,
            "0x0000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000FFFF",
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            CKB_QUOTE,
            INVITED_DISCOUNT,
            timestamp,
        );
        template.push_pre_account_cell(
            cell_data,
            Some((1, 0, entity)),
            util::gen_register_fee(8, true),
            Source::Output,
        );

        template.as_json()
    }
);
