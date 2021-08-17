use super::common::init;
use crate::util;
use crate::util::{constants::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;

challenge_with_generator!(
    challenge_pre_register_preserved_account,
    Error::AccountIsPreserved,
    || {
        let (mut template, account, timestamp) = init("microsoft.bit");
        template.push_config_cell_derived_by_account("microsoft", true, 0, Source::CellDep);

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
            util::gen_register_fee(9, true),
            Source::Output,
        );

        template.as_json()
    }
);

test_with_generator!(test_pre_register_preserved_account_with_super_lock, || {
    let (mut template, account, timestamp) = init("microsoft.bit");
    template.push_config_cell_derived_by_account("microsoft", true, 0, Source::CellDep);

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
        util::gen_register_fee(9, true),
        Source::Output,
    );

    template.as_json()
});

// TODO Need optimize with release version.
// #[test]
// fn challenge_pre_register_preserved_account() {
//     let lines = util::read_lines("preserved_accounts.txt")
//         .expect("Expect file ./data/preserved_accounts.txt exist.");
//     for line in lines {
//         if let Ok(account) = line {
//             let account_length = account.chars().count();
//             if !account.is_empty() && account_length > ACCOUNT_RELEASED_LENGTH {
//                 let account_with_suffix = account.clone() + ".bit";
//                 let (mut template, _, timestamp) = init(&account_with_suffix);
//                 template.push_config_cell_derived_by_account(&account, true, 0, Source::CellDep);
//
//                 let (cell_data, entity) = template.gen_pre_account_cell_data(
//                     &account_with_suffix,
//                     "0x0000000000000000000000000000000000002222",
//                     "0x000000000000000000000000000000000000FFFF",
//                     "0x0000000000000000000000000000000000001111",
//                     "0x0000000000000000000000000000000000002222",
//                     1000,
//                     500,
//                     timestamp,
//                 );
//                 template.push_pre_account_cell(
//                     cell_data,
//                     Some((1, 0, entity)),
//                     util::gen_register_fee(account_length, true),
//                     Source::Output,
//                 );
//
//                 let mut parser = TemplateParser::from_data(Context::default(), template.as_json());
//                 parser.parse();
//
//                 let ret = parser.execute_tx_directly();
//                 match ret {
//                     Ok(_) => {
//                         // println!("{}", serde_json::to_string_pretty(&template).unwrap());
//                         panic!(
//                             "The test should failed with error code: {}, but it returns Ok.",
//                             Error::AccountIsPreserved as i8
//                         )
//                     }
//                     Err(err) => {
//                         let msg = err.to_string();
//                         println!("Error message: {}", msg);
//
//                         let search =
//                             format!("ValidationFailure({})", Error::AccountIsPreserved as i8);
//                         assert!(
//                             msg.contains(search.as_str()),
//                             "The test should failed with error code: {}",
//                             Error::AccountIsPreserved as i8
//                         );
//                     }
//                 }
//             }
//         }
//     }
// }
