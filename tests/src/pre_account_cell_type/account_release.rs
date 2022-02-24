use super::common::init;
use crate::util::{self, constants::*, error::Error, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types_std::constants::Source;

test_with_generator!(test_pre_register_account_registrable, || {
    // This is one of the shortest registrable accounts for now, it only contains 4 chars.
    let (mut template, account, timestamp) = init("0j7p.bit");
    template.push_config_cell_derived_by_account("0j7p", Source::CellDep);

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
        util::gen_register_fee(4, true),
        Source::Output,
    );

    template.as_json()
});

test_with_generator!(test_pre_register_account_not_registrable_with_super_lock, || {
    // This is not a registrable account, it only contains 3 chars.
    let (mut template, account, timestamp) = init("mc7.bit");
    template.push_config_cell_derived_by_account("mc7", Source::CellDep);

    // BUT! it can be registered by super lock.

    // 0x0000000000000000000000000000000000000000 is the super lock in dev environment.
    template.push_signall_cell("0x0000000000000000000000000000000000000000", 0, Source::Input);

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
        util::gen_register_fee(3, true),
        Source::Output,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_pre_register_account_not_registrable,
    Error::AccountStillCanNotBeRegister,
    || {
        // This is not a registrable account, it only contains 4 chars.
        let (mut template, account, timestamp) = init("mc7.bit");
        template.push_config_cell_derived_by_account("mc7", Source::CellDep);

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
            util::gen_register_fee(3, true),
            Source::Output,
        );

        template.as_json()
    }
);

test_with_generator!(test_pre_register_account_released, || {
    // This is a registrable account, because its first 4 bytes is [44, 174, 113, 59].
    let (mut template, account, timestamp) = init("fzmb7eku.bit");
    template.push_config_cell_derived_by_account("fzmb7eku", Source::CellDep);

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
});

test_with_generator!(test_pre_register_account_released_2, || {
    // The length of account is 10, it should skip the release status check.
    let (mut template, account, timestamp) = init("1234567890.bit");
    template.push_config_cell_derived_by_account("1234567890", Source::CellDep);

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
        util::gen_register_fee(10, true),
        Source::Output,
    );

    template.as_json()
});

test_with_generator!(test_pre_register_account_unreleased_with_super_lock, || {
    // This account is not registrable, because its first 4 bytes in u32 is bigger than 3435973836.
    let (mut template, account, timestamp) = init("g0xhlqew.bit");
    template.push_config_cell_derived_by_account("g0xhlqew", Source::CellDep);

    // BUT! it can be registered by super lock.

    // 0x0000000000000000000000000000000000000000 is the super lock in dev environment.
    template.push_signall_cell("0x0000000000000000000000000000000000000000", 0, Source::Input);

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
});

challenge_with_generator!(
    challenge_pre_register_account_unreleased,
    Error::AccountStillCanNotBeRegister,
    || {
        // This account is not registrable, because its first 4 bytes in u32 is bigger than 3435973836.
        let (mut template, account, timestamp) = init("g0xhlqew.bit");
        template.push_config_cell_derived_by_account("g0xhlqew", Source::CellDep);

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
