use alloc::boxed::Box;
use ckb_std::{ckb_constants::Source, debug};
use core::result::Result;
use das_core::{
    assert,
    constants::ScriptType,
    data_parser,
    error::Error,
    sub_account_witness_parser::{SubAccountEditValue, SubAccountWitnessesParser},
    util, warn,
    witness_parser::WitnessesParser,
};
use das_types::{constants::*, packed::*, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running test-env ======");

    let mut parser = WitnessesParser::new()?;
    let action = match parser.parse_action_with_params()? {
        Some((action, _)) => action,
        None => return Err(Error::ActionNotSupported),
    };

    match action {
        b"test_parse_witness_entity_config" => {
            parser.parse_config(&[DataType::ConfigCellAccount])?;
        }
        b"test_parse_witness_raw_config" => {
            parser.parse_config(&[DataType::ConfigCellRecordKeyNamespace])?;
        }
        b"test_parse_witness_cells" => {
            parser.parse_config(&[DataType::ConfigCellMain])?;
            let config_main = parser.configs.main()?;
            let account_cell_type_id = config_main.type_id_table().account_cell();
            let account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, Source::CellDep)?;

            parser.parse_cell()?;

            let (version, _, mol_bytes) =
                parser.verify_and_get(DataType::AccountCellData, account_cells[0], Source::CellDep)?;
            let entity = Box::new(
                AccountCellData::from_slice(mol_bytes.as_reader().raw_data()).map_err(|_| {
                    warn!("Decoding AccountCellData failed");
                    Error::WitnessEntityDecodingError
                })?,
            );
            let _entity_reader = entity.as_reader();

            assert!(
                version == 3,
                Error::UnittestError,
                "The version in witness should be 3 ."
            );
        }
        b"test_parse_sub_account_witness_empty" => {
            SubAccountWitnessesParser::new()?;
        }
        b"test_parse_sub_account_witness_create" => {
            let sub_account_parser = SubAccountWitnessesParser::new()?;

            let witness_0 = sub_account_parser.get(0).expect("Should exist").expect("Should be Ok");
            let witness_1 = sub_account_parser.get(1).expect("Should exist").expect("Should be Ok");
            let witness_2 = sub_account_parser.get(2).expect("Should exist").expect("Should be Ok");

            assert!(
                witness_0.prev_root != witness_0.current_root
                    && witness_1.prev_root != witness_1.current_root
                    && witness_2.prev_root != witness_2.current_root,
                Error::UnittestError,
                "The prev_root and current_root in witnesses should not be the same."
            );

            assert!(
                witness_0.current_root == witness_1.prev_root && witness_1.current_root == witness_2.prev_root,
                Error::UnittestError,
                "The roots should be sequential."
            );

            assert!(
                witness_0.edit_key.is_empty(),
                Error::UnittestError,
                "The edit_key field should be empty."
            );
            match witness_0.edit_value {
                SubAccountEditValue::None => {}
                _ => {
                    warn!("The edit_key field should be empty");
                    return Err(Error::UnittestError);
                }
            }
        }
        b"test_parse_sub_account_witness_edit" => {
            let sub_account_parser = SubAccountWitnessesParser::new()?;

            let witness_0 = sub_account_parser.get(0).expect("Should exist").expect("Should be Ok");
            let witness_1 = sub_account_parser.get(1).expect("Should exist").expect("Should be Ok");
            let witness_2 = sub_account_parser.get(2).expect("Should exist").expect("Should be Ok");

            assert!(
                witness_0.prev_root != witness_0.current_root
                    && witness_1.prev_root != witness_1.current_root
                    && witness_2.prev_root != witness_2.current_root,
                Error::UnittestError,
                "The prev_root and current_root in witnesses should not be the same."
            );

            assert!(
                witness_0.current_root == witness_1.prev_root && witness_1.current_root == witness_2.prev_root,
                Error::UnittestError,
                "The roots should be sequential."
            );

            assert!(
                &witness_0.edit_key == b"expired_at",
                Error::UnittestError,
                "The edit_key field should be expired_at ."
            );
            match &witness_0.edit_value {
                SubAccountEditValue::ExpiredAt(val) => {
                    let expired_at = u64::from(val.as_reader());
                    assert!(
                        expired_at == u64::MAX,
                        Error::UnittestError,
                        "The edit_value should be u64::MAX"
                    );
                }
                _ => {
                    warn!("The edit_value field should be type of SubAccountEditValue::ExpiredAt .");
                    return Err(Error::UnittestError);
                }
            }

            assert!(
                &witness_1.edit_key == b"owner",
                Error::UnittestError,
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
                    return Err(Error::UnittestError);
                }
            }

            assert!(
                &witness_2.edit_key == b"records",
                Error::UnittestError,
                "The edit_key field should be records ."
            );
            match &witness_2.edit_value {
                SubAccountEditValue::Records(val) => {
                    assert!(
                        val.len() == 1,
                        Error::UnittestError,
                        "The edit_value should contains one record."
                    );
                }
                _ => {
                    warn!("The edit_value field should be type of SubAccountEditValue::Records .");
                    return Err(Error::UnittestError);
                }
            }
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}
