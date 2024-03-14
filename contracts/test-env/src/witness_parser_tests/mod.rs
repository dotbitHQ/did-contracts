use alloc::boxed::Box;
use das_core::constants::ScriptType;
use das_types::constants::Source;
use witness_parser::traits::WitnessQueryable;
use witness_parser::types::CellMeta;
use core::result::Result;

use das_types::packed as das_packed;
use das_core::{error::*, warn};
use das_core::error::ScriptError;
use das_core::config::Config;
use witness_parser::WitnessesParserV1;
use das_core::util;

pub fn test_witness_parser_get_entity_by_cell_meta() -> Result<(), Box<dyn ScriptError>> {
    let parser = WitnessesParserV1::get_instance();
    let config_main = Config::get_instance().main()?;
    let source = Source::CellDep;

    let account_cell_type_id = config_main.type_id_table().account_cell();
    let account_cells = util::find_cells_by_type_id(ScriptType::Type, account_cell_type_id, source.into())?;

    let cell_meta = CellMeta::new(account_cells[0], source.into());

    let entity = parser
        .get_entity_by_cell_meta::<das_packed::AccountCellDataV3>(cell_meta)
        .map_err(|_| {
            warn!("{:?}[{}] Decoding AccountCellDataV3 failed", source, account_cells[0]);
            ErrorCode::WitnessEntityDecodingError
        })?;
    let _entity_reader = entity.as_reader();

    // assert!(
    //     version == 3,
    //     ErrorCode::UnittestError,
    //     "The version in witness should be 3 ."
    // );

    Ok(())
}
