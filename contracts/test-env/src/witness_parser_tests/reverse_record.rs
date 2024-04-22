use alloc::boxed::Box;
use alloc::string::ToString;
use core::result::Result;

use das_core::config::Config;
use das_core::error::{ErrorCode, ScriptError};
use das_core::witness_parser::reverse_record::ReverseRecordWitnessesParser;
use das_core::{code_to_error, das_assert};
use das_types::constants::{DasLockType, ReverseRecordAction};

pub fn test_parse_reverse_record_witness_empty() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    ReverseRecordWitnessesParser::new(&config_main)?;

    Ok(())
}

pub fn test_parse_reverse_record_witness_update_only() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let witness_parser = ReverseRecordWitnessesParser::new(&config_main)?;
    for witness_ret in witness_parser.iter() {
        let witness = witness_ret.expect("Should be Ok");

        das_assert!(
            witness.version == 1,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.veresion should be 2."
        );

        das_assert!(
            witness.action == ReverseRecordAction::Update,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.action should be {}.",
            ReverseRecordAction::Update.to_string()
        );

        das_assert!(
            witness.sign_type == DasLockType::CKBSingle,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.action should be {}.",
            DasLockType::CKBSingle.to_string()
        );
    }

    Ok(())
}

pub fn test_parse_reverse_record_witness_remove_only() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let witness_parser = ReverseRecordWitnessesParser::new(&config_main)?;
    for witness_ret in witness_parser.iter() {
        let witness = witness_ret.expect("Should be Ok");

        das_assert!(
            witness.version == 1,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.veresion should be 2."
        );

        das_assert!(
            witness.action == ReverseRecordAction::Remove,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.action should be {}.",
            ReverseRecordAction::Remove.to_string()
        );

        das_assert!(
            witness.sign_type == DasLockType::CKBSingle,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.action should be {}.",
            DasLockType::CKBSingle.to_string()
        );
    }

    Ok(())
}

pub fn test_parse_reverse_record_witness_mixed() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let witness_parser = ReverseRecordWitnessesParser::new(&config_main)?;
    for witness_ret in witness_parser.iter() {
        let witness = witness_ret.expect("Should be Ok");

        das_assert!(
            witness.version == 1,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.veresion should be 2."
        );

        das_assert!(
            witness.action == ReverseRecordAction::Update || witness.action == ReverseRecordAction::Remove,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.action should be {} or {}.",
            ReverseRecordAction::Update.to_string(),
            ReverseRecordAction::Remove.to_string()
        );

        das_assert!(
            witness.sign_type == DasLockType::CKBSingle,
            ErrorCode::UnittestError,
            "The ReverseRecordWitness.action should be {}.",
            DasLockType::CKBSingle.to_string()
        );
    }

    Ok(())
}
