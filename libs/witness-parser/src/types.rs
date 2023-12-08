use das_types::constants::{DataType, Source};

pub type Hash = [u8; 32];

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct CellMeta {
    pub index: usize,
    pub source: Source,
}

#[derive(Clone, Copy, Debug)]
pub struct WitnessMeta {
    pub index: usize,
    pub version: u32,
    pub data_type: DataType,
    pub cell_meta: CellMeta,
    pub hash_in_cell_data: Hash,
}
