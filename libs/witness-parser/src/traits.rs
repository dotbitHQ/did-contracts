#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::{DataType, TypeScript};
use molecule::prelude::Entity;

use crate::error::WitnessParserError;
use crate::types::{CellMeta, Hash};

pub trait WitnessQueryable {
    fn get_type_id(&mut self, type_script: TypeScript) -> Result<Hash, WitnessParserError>;

    fn get_entity_by_cell_meta<T: Entity>(&mut self, cell_meta: CellMeta) -> Result<T, WitnessParserError>;

    fn get_entity_by_data_type<T: Entity>(&mut self, data_type: DataType) -> Result<T, WitnessParserError>;

    fn get_raw_by_index(&mut self, index: usize) -> Result<Vec<u8>, WitnessParserError>;

    fn get_raw_by_cell_meta(&mut self, cell_meta: CellMeta) -> Result<Vec<u8>, WitnessParserError>;

    fn get_raw_by_data_type(&mut self, data_type: DataType) -> Result<Vec<u8>, WitnessParserError>;
}
