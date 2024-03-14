use alloc::boxed::Box;
use alloc::string::{ToString};
use core::result::Result;

use ckb_std::debug;
use das_core::error::*;
use witness_parser::WitnessesParserV1;
use das_core::{code_to_error, warn};
use das_types::constants::*;
// use simple_ast::types as ast_types;

use crate::{config_tests, uint_tests, witness_parser_tests};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running test-env ======");

    let parser = WitnessesParserV1::get_instance();
    parser
        .init()
        .map_err(|_err| {
            debug!("_err: {:?}", _err);
            code_to_error!(ErrorCode::WitnessDataDecodingError)
        })?;

    if parser.action != Action::UnitTest {
        warn!("Action is undefined: {:?}", parser.action.to_string());
        return Err(code_to_error!(ErrorCode::ActionNotSupported));
    }
    let test_name = match &parser.action_params {
        ActionParams::TestName(name) => name,
        _ => {
            warn!("ActionParams is invalid");
            return Err(code_to_error!(ErrorCode::HardCodedError));
        }
    };

    debug!( "Route to {:?} test ...", test_name );

    match test_name.as_str() {
        "test_uint_basic_interface" => uint_tests::test_basic_interface()?,
        "test_uint_safty" => uint_tests::test_safty()?,
        "perf_uint_price_formula" => uint_tests::perf_price_formula()?,
        "test_config_account_loading" => config_tests::test_config_account_loading()?,
        "test_config_records_key_namespace_loading" => config_tests::test_config_records_key_namespace_loading()?,
        "test_witness_parser_get_entity_by_cell_meta" => {
            witness_parser_tests::test_witness_parser_get_entity_by_cell_meta()?
        }
        _ => {
            warn!("Test not found: {:?}", test_name);
            return Err(code_to_error!(ErrorCode::HardCodedError))
        }
    }

    // match parser.action {
    //     b"test_parse_witness_cells" => {
    //         let config_main = parser.configs.main()?;
    //         let account_cell_type_id = config_main.type_id_table().account_cell();
    //         let account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::CellDep)?;

    //         parser.parse_cell()?;

    //         let (version, _, mol_bytes) =
    //             parser.verify_and_get(DataType::AccountCellData, account_cells[0], Source::CellDep)?;
    //         let entity = Box::new(
    //             AccountCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
    //                 warn!("Decoding AccountCellData failed");
    //                 ErrorCode::WitnessEntityDecodingError
    //             })?,
    //         );
    //         let _entity_reader = entity.as_reader();

    //         assert!(
    //             version == 3,
    //             ErrorCode::UnittestError,
    //             "The version in witness should be 3 ."
    //         );
    //     }
    //     b"test_parse_sub_account_witness_empty" => {
    //         SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &parser.configs.main()?)?;
    //     }
    //     b"test_parse_sub_account_witness_create_only" => {
    //         let sub_account_witness_parser =
    //             SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &parser.configs.main()?)?;

    //         let lock_args = &[
    //             2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //             0, 0, 0, 0, 0, 0, 0, 255, 255,
    //         ];
    //         let sub_account_mint_sign_witness = sub_account_witness_parser
    //             .get_mint_sign(lock_args)
    //             .expect("Should exist")
    //             .expect("Should be Ok");

    //         assert!(
    //             sub_account_mint_sign_witness.version == 1,
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.version should be 1."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.sign_type == Some(DasLockType::CKBSingle),
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.sign_type should be Some(DasLockType::CKBSingle)."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.sign_role == Some(LockRole::Owner),
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.sign_role should be Some(LockRole::Owner)."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.sign_args
    //                 == vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255],
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.sign_args should be the same with the lock_args."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.expired_at > 0,
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.expired_at should be greater than 0."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.account_list_smt_root.len() == 32,
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.account_list_smt_root should be 32 bytes."
    //         );

    //         assert!(
    //             sub_account_witness_parser.len() == 3,
    //             ErrorCode::UnittestError,
    //             "There should be 3 SubAccountWitness."
    //         );

    //         assert!(
    //             sub_account_witness_parser.contains_creation == true
    //                 && sub_account_witness_parser.contains_edition == false,
    //             ErrorCode::UnittestError,
    //             "The transaction should only contains edition actions."
    //         );

    //         for witness_ret in sub_account_witness_parser.iter() {
    //             let witness = witness_ret.expect("Should be Ok");

    //             assert!(
    //                 witness.version == 2,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.veresion should be 2."
    //             );

    //             assert!(
    //                 witness.sign_expired_at == 0,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.sign_expired_at should be greater than 0."
    //             );

    //             assert!(
    //                 witness.new_root.len() == 32,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.new_root should be 32 bytes."
    //             );

    //             assert!(
    //                 witness.action == SubAccountAction::Create,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.action should be SubAccountEditValue::Create."
    //             );

    //             assert!(
    //                 witness.edit_key.is_empty(),
    //                 ErrorCode::UnittestError,
    //                 "The edit_key field should be empty."
    //             );
    //             match witness.edit_value {
    //                 SubAccountEditValue::None => {
    //                     assert!(
    //                         !witness.edit_value_bytes.is_empty(),
    //                         ErrorCode::UnittestError,
    //                         "The SubAccountWitness.edit_value_bytes should not be empty."
    //                     );
    //                 }
    //                 _ => {
    //                     warn!("The edit_key field should be empty");
    //                     return Err(code_to_error!(ErrorCode::UnittestError));
    //                 }
    //             }
    //         }
    //     }
    //     b"test_parse_sub_account_witness_edit_only" => {
    //         let sub_account_witness_parser =
    //             SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &parser.configs.main()?)?;

    //         assert!(
    //             sub_account_witness_parser.len() == 3,
    //             ErrorCode::UnittestError,
    //             "There should be 3 SubAccountWitness."
    //         );

    //         assert!(
    //             sub_account_witness_parser.contains_creation == false
    //                 && sub_account_witness_parser.contains_edition == true,
    //             ErrorCode::UnittestError,
    //             "The transaction should only contains edition actions."
    //         );

    //         for witness_ret in sub_account_witness_parser.iter() {
    //             let witness = witness_ret.expect("Should be Ok");

    //             assert!(
    //                 witness.version == 2,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.veresion should be 2."
    //             );

    //             assert!(
    //                 witness.sign_expired_at > 0,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.sign_expired_at should be greater than 0."
    //             );

    //             assert!(
    //                 witness.new_root.len() == 32,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.new_root should be 32 bytes."
    //             );

    //             assert!(
    //                 witness.action == SubAccountAction::Edit,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.action should be SubAccountEditValue::Edit."
    //             );

    //             assert!(
    //                 !witness.edit_key.is_empty(),
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.edit_key field should not be empty."
    //             );
    //         }

    //         let witness_0 = sub_account_witness_parser.get(0).unwrap().unwrap();
    //         assert!(
    //             &witness_0.edit_key == b"expired_at",
    //             ErrorCode::UnittestError,
    //             "The edit_key field should be expired_at ."
    //         );
    //         match &witness_0.edit_value {
    //             SubAccountEditValue::ExpiredAt(expired_at) => {
    //                 assert!(
    //                     expired_at.to_owned() == u64::MAX,
    //                     ErrorCode::UnittestError,
    //                     "The edit_value should be u64::MAX"
    //                 );
    //             }
    //             _ => {
    //                 warn!("The edit_value field should be type of SubAccountEditValue::ExpiredAt .");
    //                 return Err(code_to_error!(ErrorCode::UnittestError));
    //             }
    //         }

    //         let witness_1 = sub_account_witness_parser.get(1).unwrap().unwrap();
    //         assert!(
    //             &witness_1.edit_key == b"owner",
    //             ErrorCode::UnittestError,
    //             "The edit_key field should be owner ."
    //         );
    //         match &witness_1.edit_value {
    //             SubAccountEditValue::Owner(val) => {
    //                 data_parser::das_lock_args::get_owner_type(val);
    //                 data_parser::das_lock_args::get_owner_lock_args(val);
    //                 data_parser::das_lock_args::get_manager_type(val);
    //                 data_parser::das_lock_args::get_manager_lock_args(val);
    //             }
    //             _ => {
    //                 warn!("The edit_value field should be type of SubAccountEditValue::Owner .");
    //                 return Err(code_to_error!(ErrorCode::UnittestError));
    //             }
    //         }

    //         let witness_2 = sub_account_witness_parser.get(2).unwrap().unwrap();
    //         assert!(
    //             &witness_2.edit_key == b"records",
    //             ErrorCode::UnittestError,
    //             "The edit_key field should be records ."
    //         );
    //         match &witness_2.edit_value {
    //             SubAccountEditValue::Records(val) => {
    //                 assert!(
    //                     val.len() == 1,
    //                     ErrorCode::UnittestError,
    //                     "The edit_value should contains one record."
    //                 );
    //             }
    //             _ => {
    //                 warn!("The edit_value field should be type of SubAccountEditValue::Records .");
    //                 return Err(code_to_error!(ErrorCode::UnittestError));
    //             }
    //         }
    //     }
    //     b"test_parse_sub_account_witness_mixed" => {
    //         let sub_account_witness_parser =
    //             SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &parser.configs.main()?)?;

    //         let lock_args = &[
    //             2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //             0, 0, 0, 0, 0, 0, 0, 255, 255,
    //         ];
    //         let sub_account_mint_sign_witness = sub_account_witness_parser
    //             .get_mint_sign(lock_args)
    //             .expect("Should exist")
    //             .expect("Should be Ok");

    //         assert!(
    //             sub_account_mint_sign_witness.version == 1,
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.version should be 1."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.expired_at > 0,
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.expired_at should be greater than 0."
    //         );

    //         assert!(
    //             sub_account_mint_sign_witness.account_list_smt_root.len() == 32,
    //             ErrorCode::UnittestError,
    //             "The SubAccountMintSignWitness.account_list_smt_root should be 32 bytes."
    //         );

    //         assert!(
    //             sub_account_witness_parser.len() == 4,
    //             ErrorCode::UnittestError,
    //             "There should be 4 SubAccountWitness."
    //         );

    //         assert!(
    //             sub_account_witness_parser.contains_creation == true
    //                 && sub_account_witness_parser.contains_edition == true,
    //             ErrorCode::UnittestError,
    //             "The transaction should only contains edition actions."
    //         );

    //         for witness_ret in sub_account_witness_parser.iter() {
    //             let witness = witness_ret.expect("Should be Ok");

    //             assert!(
    //                 witness.version == 2,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.veresion should be 2."
    //             );

    //             assert!(
    //                 witness.new_root.len() == 32,
    //                 ErrorCode::UnittestError,
    //                 "The SubAccountWitness.new_root should be 32 bytes."
    //             );

    //             match witness.action {
    //                 SubAccountAction::Create => {
    //                     assert!(
    //                         witness.sign_expired_at == 0,
    //                         ErrorCode::UnittestError,
    //                         "The SubAccountWitness.sign_expired_at should be greater than 0."
    //                     );

    //                     assert!(
    //                         witness.edit_key.is_empty(),
    //                         ErrorCode::UnittestError,
    //                         "The SubAccountWitness.edit_key field should not be empty."
    //                     );
    //                 }
    //                 SubAccountAction::Edit => {
    //                     assert!(
    //                         witness.sign_expired_at > 0,
    //                         ErrorCode::UnittestError,
    //                         "The SubAccountWitness.sign_expired_at should be greater than 0."
    //                     );

    //                     assert!(
    //                         !witness.edit_key.is_empty(),
    //                         ErrorCode::UnittestError,
    //                         "The SubAccountWitness.edit_key field should not be empty."
    //                     );
    //                 }
    //                 _ => {
    //                     assert!(
    //                         false,
    //                         ErrorCode::UnittestError,
    //                         "This action {:?} has not been implemented.", witness.action
    //                     );
    //                 }
    //             }
    //         }
    //     }
    //     b"test_parser_sub_account_rules_witness_empty" => {
    //         let sub_account_witness_parser =
    //             SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &parser.configs.main()?)?;
    //         sub_account_witness_parser.get_rules(&[0u8; 10], DataType::SubAccountPriceRule)?;
    //     }
    //     b"test_parser_sub_account_rules_witness" => {
    //         let sub_account_witness_parser =
    //             SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &parser.configs.main()?)?;
    //         let (_, rules) = sub_account_witness_parser.get_rules(&[0u8; 10], DataType::SubAccountPriceRule)?;

    //         assert!(rules.is_some(), ErrorCode::UnittestError, "This rules should be some.");

    //         let rules = rules.unwrap();
    //         matches!(rules[0].clone(), ast_types::SubAccountRule {
    //             index: 0,
    //             name: x,
    //             note: y,
    //             price: 100_000_000,
    //             status: ast_types::SubAccountRuleStatus::On,
    //             ast: ast_types::Expression::Operator(ast_types::OperatorExpression {
    //                 symbol: ast_types::SymbolType::And,
    //                 expressions: _
    //             })
    //         } if x == String::from("Price of 1 Charactor Emoji DID") && y == String::new());

    //         // let expressions = match rules[0].ast.clone() {
    //         //     ast_types::Expression::Operator(ast_types::OperatorExpression {
    //         //         symbol: ast_types::SymbolType::And,
    //         //         expressions: expressions
    //         //     }) => {
    //         //         // TODO
    //         //     },
    //         //     _ => panic!("The rules[0].ast should be OperatorExpression.")
    //         // };
    //     }
    //     b"test_parse_reverse_record_witness_empty" => {
    //         ReverseRecordWitnessesParser::new(&parser.configs.main()?)?;
    //     }
    //     b"test_parse_reverse_record_witness_update_only" => {
    //         let witness_parser = ReverseRecordWitnessesParser::new(&parser.configs.main()?)?;
    //         for witness_ret in witness_parser.iter() {
    //             let witness = witness_ret.expect("Should be Ok");

    //             assert!(
    //                 witness.version == 1,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.veresion should be 2."
    //             );

    //             assert!(
    //                 witness.action == ReverseRecordAction::Update,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.action should be {}.",
    //                 ReverseRecordAction::Update.to_string()
    //             );

    //             assert!(
    //                 witness.sign_type == DasLockType::CKBSingle,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.action should be {}.",
    //                 DasLockType::CKBSingle.to_string()
    //             );
    //         }
    //     }
    //     b"test_parse_reverse_record_witness_remove_only" => {
    //         let witness_parser = ReverseRecordWitnessesParser::new(&parser.configs.main()?)?;
    //         for witness_ret in witness_parser.iter() {
    //             let witness = witness_ret.expect("Should be Ok");

    //             assert!(
    //                 witness.version == 1,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.veresion should be 2."
    //             );

    //             assert!(
    //                 witness.action == ReverseRecordAction::Remove,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.action should be {}.",
    //                 ReverseRecordAction::Remove.to_string()
    //             );

    //             assert!(
    //                 witness.sign_type == DasLockType::CKBSingle,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.action should be {}.",
    //                 DasLockType::CKBSingle.to_string()
    //             );
    //         }
    //     }
    //     b"test_parse_reverse_record_witness_mixed" => {
    //         let witness_parser = ReverseRecordWitnessesParser::new(&parser.configs.main()?)?;
    //         for witness_ret in witness_parser.iter() {
    //             let witness = witness_ret.expect("Should be Ok");

    //             assert!(
    //                 witness.version == 1,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.veresion should be 2."
    //             );

    //             assert!(
    //                 witness.action == ReverseRecordAction::Update || witness.action == ReverseRecordAction::Remove,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.action should be {} or {}.",
    //                 ReverseRecordAction::Update.to_string(),
    //                 ReverseRecordAction::Remove.to_string()
    //             );

    //             assert!(
    //                 witness.sign_type == DasLockType::CKBSingle,
    //                 ErrorCode::UnittestError,
    //                 "The ReverseRecordWitness.action should be {}.",
    //                 DasLockType::CKBSingle.to_string()
    //             );
    //         }
    //     }
    // }

    Ok(())
}
