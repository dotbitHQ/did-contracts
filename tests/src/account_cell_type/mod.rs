mod account_transfer;
mod common;
mod edit_manager;
mod edit_records;
mod force_recover_account_status;
mod init_account_chain;

// #[test]
// fn gen_account_renew() {
//     let (mut template, timestamp) = init("renew_account", None);
//
//     template.push_contract_cell("income-cell-type", false);
//
//     template.push_oracle_cell(1, OracleCellType::Quote, 1000);
//     template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::CellDep);
//
//     let account = "das00001.bit";
//     let next_account = "das00014.bit";
//     let registered_at = timestamp - 86400;
//     let expired_at = timestamp + 31536000 - 86400;
//
//     // inputs
//     let (cell_data, old_entity) =
//         template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
//     template.push_account_cell::<AccountCellData>(
//         "0x0000000000000000000000000000000000001111",
//         "0x0000000000000000000000000000000000002222",
//         cell_data,
//         None,
//         20_000_000_000,
//         Source::Input,
//     );
//
//     let income_records = vec![IncomeRecordParam {
//         belong_to: "0x0000000000000000000000000000000000000000".to_string(),
//         capacity: 20_000_000_000,
//     }];
//     let (cell_data, entity) =
//         template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
//     template.push_income_cell(cell_data, Some((1, 1, entity)), 20_000_000_000, Source::Input);
//
//     // outputs
//     let (cell_data, new_entity) = template.gen_account_cell_data(
//         account,
//         next_account,
//         registered_at,
//         expired_at + 86400 * 365,
//         0,
//         0,
//         0,
//         None,
//     );
//     template.push_account_cell::<AccountCellData>(
//         "0x0000000000000000000000000000000000001111",
//         "0x0000000000000000000000000000000000002222",
//         cell_data,
//         None,
//         20_000_000_000,
//         Source::Output,
//     );
//     template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
//         DataType::AccountCellData,
//         Some((2, 0, new_entity)),
//         Some((2, 0, old_entity)),
//         None,
//     );
//
//     let income_records = vec![
//         IncomeRecordParam {
//             belong_to: "0x0000000000000000000000000000000000000000".to_string(),
//             capacity: 20_000_000_000,
//         },
//         // Profit to DAS
//         IncomeRecordParam {
//             belong_to: "0x0300000000000000000000000000000000000000".to_string(),
//             capacity: 500_000_000_000,
//         },
//     ];
//     let (cell_data, entity) =
//         template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
//     template.push_income_cell(cell_data, Some((1, 1, entity)), 520_000_000_000, Source::Output);
//
//     template.write_template("account_renew_account.json");
// }
//
// test_with_template!(test_account_renew, "account_renew_account.json");
//
// challenge_with_generator!(
//     challenge_account_renew_with_das_lock,
//     Error::InvalidTransactionStructure,
//     || {
//         let (mut template, timestamp) = init("renew_account", None);
//
//         template.push_contract_cell("income-cell-type", false);
//         template.push_contract_cell("balance-cell-type", false);
//
//         template.push_oracle_cell(1, OracleCellType::Quote, 1000);
//         template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::CellDep);
//
//         let account = "das00001.bit";
//         let next_account = "das00014.bit";
//         let registered_at = timestamp - 86400;
//         let expired_at = timestamp + 31536000 - 86400;
//
//         // inputs
//         let (cell_data, old_entity) =
//             template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
//         template.push_account_cell::<AccountCellData>(
//             "0x0000000000000000000000000000000000001111",
//             "0x0000000000000000000000000000000000002222",
//             cell_data,
//             None,
//             20_000_000_000,
//             Source::Input,
//         );
//
//         template.push_das_lock_cell(
//             "0x030000000000000000000000000000000000004444",
//             500_000_000_000,
//             Source::Input,
//             None,
//         );
//
//         let income_records = vec![IncomeRecordParam {
//             belong_to: "0x0000000000000000000000000000000000000000".to_string(),
//             capacity: 20_000_000_000,
//         }];
//         let (cell_data, entity) =
//             template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
//         template.push_income_cell(cell_data, Some((1, 1, entity)), 20_000_000_000, Source::Input);
//
//         // outputs
//         let (cell_data, new_entity) = template.gen_account_cell_data(
//             account,
//             next_account,
//             registered_at,
//             expired_at + 86400 * 365,
//             0,
//             0,
//             0,
//             None,
//         );
//         template.push_account_cell::<AccountCellData>(
//             "0x0000000000000000000000000000000000001111",
//             "0x0000000000000000000000000000000000002222",
//             cell_data,
//             None,
//             20_000_000_000,
//             Source::Output,
//         );
//         template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
//             DataType::AccountCellData,
//             Some((2, 0, new_entity)),
//             Some((2, 0, old_entity)),
//             None,
//         );
//
//         let income_records = vec![
//             IncomeRecordParam {
//                 belong_to: "0x0000000000000000000000000000000000000000".to_string(),
//                 capacity: 20_000_000_000,
//             },
//             // Profit to DAS
//             IncomeRecordParam {
//                 belong_to: "0x0300000000000000000000000000000000000000".to_string(),
//                 capacity: 500_000_000_000,
//             },
//         ];
//         let (cell_data, entity) =
//             template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
//         template.push_income_cell(cell_data, Some((1, 1, entity)), 520_000_000_000, Source::Output);
//
//         template.as_json()
//     }
// );
//
// #[test]
// fn gen_account_recycle_expired_account_by_keeper() {
//     let (mut template, timestamp) = init("recycle_expired_account_by_keeper", None);
//
//     let account = "das00001.bit";
//     let next_account = "das00014.bit";
//     let registered_at = timestamp - 86400 * (365 + 30); // Register at 1 year and 1 month before
//     let expired_at = timestamp - 86400 * 30 - 1; // Expired at 1 month + 1 second before
//
//     let (cell_data, old_entity) =
//         template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
//     template.push_account_cell::<AccountCellData>(
//         "0x0000000000000000000000000000000000001111",
//         "0x0000000000000000000000000000000000002222",
//         cell_data,
//         Some((1, 0, old_entity)),
//         21_200_000_000,
//         Source::Input,
//     );
//
//     template.push_signall_cell(
//         "0x0000000000000000000000000000000000001111",
//         21_200_000_000,
//         Source::Output,
//     );
//
//     template.write_template("account_recycle_expired_account_by_keeper.json");
// }
//
// test_with_template!(
//     test_account_recycle_expired_account_by_keeper,
//     "account_recycle_expired_account_by_keeper.json"
// );
