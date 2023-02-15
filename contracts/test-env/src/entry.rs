use alloc::boxed::Box;
use alloc::string::ToString;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::debug;
use das_core::constants::ScriptType;
use das_core::error::*;
use das_core::witness_parser::reverse_record::*;
use das_core::witness_parser::sub_account::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, code_to_error, data_parser, util, warn};
use das_types::constants::*;
use das_types::packed::*;
use das_types::prelude::*;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running test-env ======");

    let mut parser = WitnessesParser::new()?;
    let action = match parser.parse_action_with_params()? {
        Some((action, _)) => action,
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );

    match action {
        b"test_parse_witness_entity_config" => {
            parser.configs.account()?;
        }
        b"test_parse_witness_raw_config" => {
            parser.configs.record_key_namespace()?;
        }
        b"test_parse_witness_cells" => {
            let config_main = parser.configs.main()?;
            let account_cell_type_id = config_main.type_id_table().account_cell();
            let account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::CellDep)?;

            parser.parse_cell()?;

            let (version, _, mol_bytes) =
                parser.verify_and_get(DataType::AccountCellData, account_cells[0], Source::CellDep)?;
            let entity = Box::new(
                AccountCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                    warn!("Decoding AccountCellData failed");
                    ErrorCode::WitnessEntityDecodingError
                })?,
            );
            let _entity_reader = entity.as_reader();

            assert!(
                version == 3,
                ErrorCode::UnittestError,
                "The version in witness should be 3 ."
            );
        }
        b"test_parse_sub_account_witness_empty" => {
            SubAccountWitnessesParser::new()?;
        }
        b"test_parse_sub_account_witness_create_only" => {
            let sub_account_witness_parser = SubAccountWitnessesParser::new()?;

            let lock_args = &[
                2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 255,
            ];
            let sub_account_mint_sign_witness = sub_account_witness_parser
                .get_mint_sign(lock_args)
                .expect("Should exist")
                .expect("Should be Ok");

            assert!(
                sub_account_mint_sign_witness.version == 1,
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.version should be 1."
            );

            assert!(
                sub_account_mint_sign_witness.sign_type == Some(DasLockType::CKBSingle),
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.sign_type should be Some(DasLockType::CKBSingle)."
            );

            assert!(
                sub_account_mint_sign_witness.sign_role == Some(LockRole::Owner),
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.sign_role should be Some(LockRole::Owner)."
            );

            assert!(
                sub_account_mint_sign_witness.sign_args
                    == vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255],
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.sign_args should be the same with the lock_args."
            );

            assert!(
                sub_account_mint_sign_witness.expired_at > 0,
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.expired_at should be greater than 0."
            );

            assert!(
                sub_account_mint_sign_witness.account_list_smt_root.len() == 32,
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.account_list_smt_root should be 32 bytes."
            );

            assert!(
                sub_account_witness_parser.len() == 3,
                ErrorCode::UnittestError,
                "There should be 3 SubAccountWitness."
            );

            assert!(
                sub_account_witness_parser.contains_creation == true
                    && sub_account_witness_parser.contains_edition == false,
                ErrorCode::UnittestError,
                "The transaction should only contains edition actions."
            );

            for witness_ret in sub_account_witness_parser.iter() {
                let witness = witness_ret.expect("Should be Ok");

                assert!(
                    witness.version == 2,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.veresion should be 2."
                );

                assert!(
                    witness.sign_expired_at == 0,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.sign_expired_at should be greater than 0."
                );

                assert!(
                    witness.new_root.len() == 32,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.new_root should be 32 bytes."
                );

                assert!(
                    witness.action == SubAccountAction::Create,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.action should be SubAccountEditValue::Create."
                );

                assert!(
                    witness.edit_key.is_empty(),
                    ErrorCode::UnittestError,
                    "The edit_key field should be empty."
                );
                match witness.edit_value {
                    SubAccountEditValue::None => {
                        assert!(
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
        }
        b"test_parse_sub_account_witness_edit_only" => {
            let sub_account_witness_parser = SubAccountWitnessesParser::new()?;

            assert!(
                sub_account_witness_parser.len() == 3,
                ErrorCode::UnittestError,
                "There should be 3 SubAccountWitness."
            );

            assert!(
                sub_account_witness_parser.contains_creation == false
                    && sub_account_witness_parser.contains_edition == true,
                ErrorCode::UnittestError,
                "The transaction should only contains edition actions."
            );

            for witness_ret in sub_account_witness_parser.iter() {
                let witness = witness_ret.expect("Should be Ok");

                assert!(
                    witness.version == 2,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.veresion should be 2."
                );

                assert!(
                    witness.sign_expired_at > 0,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.sign_expired_at should be greater than 0."
                );

                assert!(
                    witness.new_root.len() == 32,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.new_root should be 32 bytes."
                );

                assert!(
                    witness.action == SubAccountAction::Edit,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.action should be SubAccountEditValue::Edit."
                );

                assert!(
                    !witness.edit_key.is_empty(),
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.edit_key field should not be empty."
                );
            }

            let witness_0 = sub_account_witness_parser.get(0).unwrap().unwrap();
            assert!(
                &witness_0.edit_key == b"expired_at",
                ErrorCode::UnittestError,
                "The edit_key field should be expired_at ."
            );
            match &witness_0.edit_value {
                SubAccountEditValue::ExpiredAt(val) => {
                    let expired_at = u64::from(val.as_reader());
                    assert!(
                        expired_at == u64::MAX,
                        ErrorCode::UnittestError,
                        "The edit_value should be u64::MAX"
                    );
                }
                _ => {
                    warn!("The edit_value field should be type of SubAccountEditValue::ExpiredAt .");
                    return Err(code_to_error!(ErrorCode::UnittestError));
                }
            }

            let witness_1 = sub_account_witness_parser.get(1).unwrap().unwrap();
            assert!(
                &witness_1.edit_key == b"owner",
                ErrorCode::UnittestError,
                "The edit_key field should be owner ."
            );
            match &witness_1.edit_value {
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

            let witness_2 = sub_account_witness_parser.get(2).unwrap().unwrap();
            assert!(
                &witness_2.edit_key == b"records",
                ErrorCode::UnittestError,
                "The edit_key field should be records ."
            );
            match &witness_2.edit_value {
                SubAccountEditValue::Records(val) => {
                    assert!(
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
        }
        b"test_parse_sub_account_witness_mixed" => {
            let sub_account_witness_parser = SubAccountWitnessesParser::new()?;

            let lock_args = &[
                2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 255,
            ];
            let sub_account_mint_sign_witness = sub_account_witness_parser
                .get_mint_sign(lock_args)
                .expect("Should exist")
                .expect("Should be Ok");

            assert!(
                sub_account_mint_sign_witness.version == 1,
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.version should be 1."
            );

            assert!(
                sub_account_mint_sign_witness.expired_at > 0,
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.expired_at should be greater than 0."
            );

            assert!(
                sub_account_mint_sign_witness.account_list_smt_root.len() == 32,
                ErrorCode::UnittestError,
                "The SubAccountMintSignWitness.account_list_smt_root should be 32 bytes."
            );

            assert!(
                sub_account_witness_parser.len() == 4,
                ErrorCode::UnittestError,
                "There should be 4 SubAccountWitness."
            );

            assert!(
                sub_account_witness_parser.contains_creation == true
                    && sub_account_witness_parser.contains_edition == true,
                ErrorCode::UnittestError,
                "The transaction should only contains edition actions."
            );

            for witness_ret in sub_account_witness_parser.iter() {
                let witness = witness_ret.expect("Should be Ok");

                assert!(
                    witness.version == 2,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.veresion should be 2."
                );

                assert!(
                    witness.new_root.len() == 32,
                    ErrorCode::UnittestError,
                    "The SubAccountWitness.new_root should be 32 bytes."
                );

                match witness.action {
                    SubAccountAction::Create => {
                        assert!(
                            witness.sign_expired_at == 0,
                            ErrorCode::UnittestError,
                            "The SubAccountWitness.sign_expired_at should be greater than 0."
                        );

                        assert!(
                            witness.edit_key.is_empty(),
                            ErrorCode::UnittestError,
                            "The SubAccountWitness.edit_key field should not be empty."
                        );
                    }
                    SubAccountAction::Edit => {
                        assert!(
                            witness.sign_expired_at > 0,
                            ErrorCode::UnittestError,
                            "The SubAccountWitness.sign_expired_at should be greater than 0."
                        );

                        assert!(
                            !witness.edit_key.is_empty(),
                            ErrorCode::UnittestError,
                            "The SubAccountWitness.edit_key field should not be empty."
                        );
                    }
                    _ => {
                        assert!(
                            false,
                            ErrorCode::UnittestError,
                            "This action {:?} has not been implemented.", witness.action
                        );
                    }
                }
            }
        }
        b"test_parse_reverse_record_witness_empty" => {
            ReverseRecordWitnessesParser::new()?;
        }
        b"test_parse_reverse_record_witness_update_only" => {
            let witness_parser = ReverseRecordWitnessesParser::new()?;
            for witness_ret in witness_parser.iter() {
                let witness = witness_ret.expect("Should be Ok");

                assert!(
                    witness.version == 1,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.veresion should be 2."
                );

                assert!(
                    witness.action == ReverseRecordAction::Update,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.action should be {}.",
                    ReverseRecordAction::Update.to_string()
                );

                assert!(
                    witness.sign_type == DasLockType::CKBSingle,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.action should be {}.",
                    DasLockType::CKBSingle.to_string()
                );
            }
        }
        b"test_parse_reverse_record_witness_remove_only" => {
            let witness_parser = ReverseRecordWitnessesParser::new()?;
            for witness_ret in witness_parser.iter() {
                let witness = witness_ret.expect("Should be Ok");

                assert!(
                    witness.version == 1,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.veresion should be 2."
                );

                assert!(
                    witness.action == ReverseRecordAction::Remove,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.action should be {}.",
                    ReverseRecordAction::Remove.to_string()
                );

                assert!(
                    witness.sign_type == DasLockType::CKBSingle,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.action should be {}.",
                    DasLockType::CKBSingle.to_string()
                );
            }
        }
        b"test_parse_reverse_record_witness_mixed" => {
            let witness_parser = ReverseRecordWitnessesParser::new()?;
            for witness_ret in witness_parser.iter() {
                let witness = witness_ret.expect("Should be Ok");

                assert!(
                    witness.version == 1,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.veresion should be 2."
                );

                assert!(
                    witness.action == ReverseRecordAction::Update || witness.action == ReverseRecordAction::Remove,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.action should be {} or {}.",
                    ReverseRecordAction::Update.to_string(),
                    ReverseRecordAction::Remove.to_string()
                );

                assert!(
                    witness.sign_type == DasLockType::CKBSingle,
                    ErrorCode::UnittestError,
                    "The ReverseRecordWitness.action should be {}.",
                    DasLockType::CKBSingle.to_string()
                );
            }
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
