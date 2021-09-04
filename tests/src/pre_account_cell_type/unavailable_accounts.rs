use super::common::init;
use crate::util;
use crate::util::{constants::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;

challenge_with_generator!(
    challenge_pre_register_unavailable_accounts,
    Error::AccountIsUnAvailable,
    || {
        let (mut template, account, timestamp) = init("thiscantr.bit");
        template.push_config_cell_derived_by_account("thiscantr", true, 0, Source::CellDep);

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
            util::gen_register_fee("thiscantr".len(), true),
            Source::Output,
        );

        template.as_json()
    }
);
