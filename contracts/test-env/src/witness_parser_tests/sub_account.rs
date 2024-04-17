use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use core::result::Result;

use das_core::config::Config;
use das_core::error::{ErrorCode, ScriptError};
use das_core::witness_parser::sub_account::{SubAccountEditValue, SubAccountWitnessesParser};
use das_core::{code_to_error, das_assert, warn};
use das_types::constants::{DasLockType, DataType, LockRole, SubAccountAction, SubAccountConfigFlag};
use das_types::data_parser;
use simple_ast::types as ast_types;

pub fn test_parse_sub_account_witness_empty() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &config_main)?;

    Ok(())
}

pub fn test_parse_sub_account_witness_create_only() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let sub_account_witness_parser = SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &config_main)?;

    let lock_args = &[
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 255, 255,
    ];
    let sub_account_mint_sign_witness = sub_account_witness_parser
        .get_mint_sign(lock_args)
        .expect("Should exist")
        .expect("Should be Ok");

    das_assert!(
        sub_account_mint_sign_witness.version == 1,
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.version should be 1."
    );

    das_assert!(
        sub_account_mint_sign_witness.sign_type == Some(DasLockType::CKBSingle),
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.sign_type should be Some(DasLockType::CKBSingle)."
    );

    das_assert!(
        sub_account_mint_sign_witness.sign_role == Some(LockRole::Owner),
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.sign_role should be Some(LockRole::Owner)."
    );

    das_assert!(
        sub_account_mint_sign_witness.sign_args == vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255],
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.sign_args should be the same with the lock_args."
    );

    das_assert!(
        sub_account_mint_sign_witness.expired_at > 0,
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.expired_at should be greater than 0."
    );

    das_assert!(
        sub_account_mint_sign_witness.account_list_smt_root.len() == 32,
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.account_list_smt_root should be 32 bytes."
    );

    das_assert!(
        sub_account_witness_parser.len() == 3,
        ErrorCode::UnittestError,
        "There should be 3 SubAccountWitness."
    );

    das_assert!(
        sub_account_witness_parser.contains_creation == true && sub_account_witness_parser.contains_edition == false,
        ErrorCode::UnittestError,
        "The transaction should only contains edition actions."
    );

    for witness_ret in sub_account_witness_parser.iter() {
        let witness = witness_ret.expect("Should be Ok");

        das_assert!(
            witness.version == 3,
            ErrorCode::UnittestError,
            "The SubAccountWitness.veresion should be 3."
        );

        das_assert!(
            witness.sign_expired_at == 0,
            ErrorCode::UnittestError,
            "The SubAccountWitness.sign_expired_at should be greater than 0."
        );

        das_assert!(
            witness.new_root.len() == 32,
            ErrorCode::UnittestError,
            "The SubAccountWitness.new_root should be 32 bytes."
        );

        das_assert!(
            witness.action == SubAccountAction::Create,
            ErrorCode::UnittestError,
            "The SubAccountWitness.action should be SubAccountEditValue::Create."
        );

        das_assert!(
            witness.edit_key.is_empty(),
            ErrorCode::UnittestError,
            "The edit_key field should be empty."
        );
        match witness.edit_value {
            SubAccountEditValue::None => {
                das_assert!(
                    !witness.edit_value_bytes.is_empty(),
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.edit_value_bytes should not be empty."
                );
            }
            _ => {
                warn!("The edit_key field should be empty");
                return Err(code_to_error!(ErrorCode::UnittestError));
            }
        }
    }

    Ok(())
}

pub fn test_parse_sub_account_witness_edit_only() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let sub_account_witness_parser = SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &config_main)?;

    das_assert!(
        sub_account_witness_parser.len() == 2,
        ErrorCode::UnittestError,
        "There should be 2 SubAccountWitness."
    );

    das_assert!(
        sub_account_witness_parser.contains_creation == false && sub_account_witness_parser.contains_edition == true,
        ErrorCode::UnittestError,
        "The transaction should only contains edition actions."
    );

    for witness_ret in sub_account_witness_parser.iter() {
        let witness = witness_ret.expect("Should be Ok");

        das_assert!(
            witness.version == 3,
            ErrorCode::UnittestError,
            "The SubAccountWitness.veresion should be 3."
        );

        das_assert!(
            witness.sign_expired_at > 0,
            ErrorCode::UnittestError,
            "The SubAccountWitness.sign_expired_at should be greater than 0."
        );

        das_assert!(
            witness.new_root.len() == 32,
            ErrorCode::UnittestError,
            "The SubAccountWitness.new_root should be 32 bytes."
        );

        das_assert!(
            witness.action == SubAccountAction::Edit,
            ErrorCode::UnittestError,
            "The SubAccountWitness.action should be SubAccountEditValue::Edit."
        );

        das_assert!(
            !witness.edit_key.is_empty(),
            ErrorCode::UnittestError,
            "The SubAccountWitness.edit_key field should not be empty."
        );
    }

    let witness_0 = sub_account_witness_parser.get(0).unwrap().unwrap();
    das_assert!(
        &witness_0.edit_key == b"owner",
        ErrorCode::UnittestError,
        "The edit_key field should be owner ."
    );
    match &witness_0.edit_value {
        SubAccountEditValue::Owner(val) => {
            data_parser::das_lock_args::get_owner_type(val);
            data_parser::das_lock_args::get_owner_lock_args(val);
            data_parser::das_lock_args::get_manager_type(val);
            data_parser::das_lock_args::get_manager_lock_args(val);
        }
        _ => {
            warn!("The edit_value field should be type of SubAccountEditValue::Owner .");
            return Err(code_to_error!(ErrorCode::UnittestError));
        }
    }

    let witness_1 = sub_account_witness_parser.get(1).unwrap().unwrap();
    das_assert!(
        &witness_1.edit_key == b"records",
        ErrorCode::UnittestError,
        "The edit_key field should be records ."
    );
    match &witness_1.edit_value {
        SubAccountEditValue::Records(val) => {
            das_assert!(
                val.len() == 1,
                ErrorCode::UnittestError,
                "The edit_value should contains one record."
            );
        }
        _ => {
            warn!("The edit_value field should be type of SubAccountEditValue::Records .");
            return Err(code_to_error!(ErrorCode::UnittestError));
        }
    }

    Ok(())
}

pub fn test_parse_sub_account_witness_mixed() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let sub_account_witness_parser = SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &config_main)?;

    let lock_args = &[
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 255, 255,
    ];
    let sub_account_mint_sign_witness = sub_account_witness_parser
        .get_mint_sign(lock_args)
        .expect("Should exist")
        .expect("Should be Ok");

    das_assert!(
        sub_account_mint_sign_witness.version == 1,
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.version should be 1."
    );

    das_assert!(
        sub_account_mint_sign_witness.expired_at > 0,
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.expired_at should be greater than 0."
    );

    das_assert!(
        sub_account_mint_sign_witness.account_list_smt_root.len() == 32,
        ErrorCode::UnittestError,
        "The SubAccountMintSignWitness.account_list_smt_root should be 32 bytes."
    );

    das_assert!(
        sub_account_witness_parser.len() == 4,
        ErrorCode::UnittestError,
        "There should be 4 SubAccountWitness."
    );

    das_assert!(
        sub_account_witness_parser.contains_creation == true && sub_account_witness_parser.contains_edition == true,
        ErrorCode::UnittestError,
        "The transaction should only contains edition actions."
    );

    for witness_ret in sub_account_witness_parser.iter() {
        let witness = witness_ret.expect("Should be Ok");

        das_assert!(
            witness.version == 3,
            ErrorCode::UnittestError,
            "The SubAccountWitness.veresion should be 3."
        );

        das_assert!(
            witness.new_root.len() == 32,
            ErrorCode::UnittestError,
            "The SubAccountWitness.new_root should be 32 bytes."
        );

        match witness.action {
            SubAccountAction::Create => {
                das_assert!(
                    witness.sign_expired_at == 0,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.sign_expired_at should be greater than 0."
                );

                das_assert!(
                    witness.edit_key.is_empty(),
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.edit_key field should not be empty."
                );
            }
            SubAccountAction::Edit => {
                das_assert!(
                    witness.sign_expired_at > 0,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.sign_expired_at should be greater than 0."
                );

                das_assert!(
                    !witness.edit_key.is_empty(),
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.edit_key field should not be empty."
                );
            }
            _ => {
                das_assert!(
                    false,
                    ErrorCode::UnittestError,
                    "This action {:?} has not been implemented.",
                    witness.action
                );
            }
        }
    }

    Ok(())
}

pub fn test_parse_sub_account_rules_witness_empty() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let sub_account_witness_parser = SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &config_main)?;

    sub_account_witness_parser.get_rules(&[0u8; 10], DataType::SubAccountPriceRule)?;

    Ok(())
}

pub fn test_parse_sub_account_rules_witness_simple() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let sub_account_witness_parser = SubAccountWitnessesParser::new(SubAccountConfigFlag::CustomRule, &config_main)?;

    let mut expected_data = vec![0u8; 50];
    expected_data.extend(hex::decode("4000016615d4645428ec").unwrap());

    let (_, rules) = sub_account_witness_parser.get_rules(&expected_data, DataType::SubAccountPriceRule)?;

    das_assert!(rules.is_some(), ErrorCode::UnittestError, "This rules should be some.");

    let rules = rules.unwrap();
    matches!(rules[0].clone(), ast_types::SubAccountRule {
        index: 0,
        name: x,
        note: y,
        price: 100_000_000,
        status: ast_types::SubAccountRuleStatus::On,
        ast: ast_types::Expression::Operator(ast_types::OperatorExpression {
            symbol: ast_types::SymbolType::And,
            expressions: _
        })
    } if x == String::from("Price of 1 Charactor Emoji DID") && y == String::new());

    Ok(())
}
