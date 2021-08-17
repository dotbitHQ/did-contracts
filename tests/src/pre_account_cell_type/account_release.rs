use super::common::init;
use crate::util;
use crate::util::{constants::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;

test_with_generator!(test_pre_register_account_registrable, || {
    // This is one of the shortest registrable accounts for now, it only contains 5 chars.
    let (mut template, account, timestamp) = init("1qoqm.bit");
    template.push_config_cell_derived_by_account("1qoqm", true, 0, Source::CellDep);

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
        util::gen_register_fee(5, true),
        Source::Output,
    );

    template.as_json()
});

test_with_generator!(
    test_pre_register_account_not_registrable_with_super_lock,
    || {
        // This is not a registrable account, it only contains 4 chars.
        let (mut template, account, timestamp) = init("nsn2.bit");
        template.push_config_cell_derived_by_account("nsn2", true, 0, Source::CellDep);

        // 0x0000000000000000000000000000000000000000 is the super lock in dev environment.
        template.push_signall_cell(
            "0x0000000000000000000000000000000000000000",
            0,
            Source::Input,
        );

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
    }
);

challenge_with_generator!(
    challenge_pre_register_account_not_registrable,
    Error::AccountStillCanNotBeRegister,
    || {
        // This is not a registrable account, it only contains 4 chars.
        let (mut template, account, timestamp) = init("e3bn.bit");
        template.push_config_cell_derived_by_account("e3bn", true, 0, Source::CellDep);

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
            util::gen_register_fee(1, true),
            Source::Output,
        );

        template.as_json()
    }
);

test_with_generator!(test_pre_register_account_released, || {
    // The first byte of hash is 200, it just equal to threshold 200.
    let (mut template, account, timestamp) = init("dh5vyto8.bit");
    template.push_config_cell_derived_by_account("dh5vyto8", true, 0, Source::CellDep);

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
    template.push_config_cell_derived_by_account("1234567890", true, 0, Source::CellDep);

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
    // The first byte of hash is 201, it just bigger than threshold 200.
    let (mut template, account, timestamp) = init("4lorcguq.bit");
    template.push_config_cell_derived_by_account("4lorcguq", true, 0, Source::CellDep);

    // 0x0000000000000000000000000000000000000000 is the super lock in dev environment.
    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        0,
        Source::Input,
    );

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
        // The first byte of hash is 201, it just bigger than threshold 200.
        let (mut template, account, timestamp) = init("kaecieg1.bit");
        template.push_config_cell_derived_by_account("kaecieg1", true, 0, Source::CellDep);

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
