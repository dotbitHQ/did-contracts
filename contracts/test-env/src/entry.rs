use alloc::boxed::Box;
use ckb_std::{ckb_constants::Source, debug};
use core::result::Result;
use das_core::{assert, constants::ScriptType, error::Error, util, warn, witness_parser::WitnessesParser};
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
            let entity_reader = entity.as_reader();

            assert!(
                version == 2,
                Error::UnittestError,
                "The version in witness should be 2 ."
            );
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}
