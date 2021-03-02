use super::constants::ScriptHashType;
use alloc::vec::Vec;
use das_types::packed::{ConfigCellMain, ConfigCellMarket, ConfigCellRegister};

#[derive(Debug)]
pub struct ScriptLiteral {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub args: Vec<u8>,
}

#[derive(Debug)]
pub struct Configs {
    pub main: Option<ConfigCellMain>,
    pub register: Option<ConfigCellRegister>,
    pub bloom_filter: Option<Vec<u8>>,
    pub market: Option<ConfigCellMarket>,
}

impl Configs {
    pub fn new() -> Self {
        Configs {
            main: None,
            register: None,
            bloom_filter: None,
            market: None,
        }
    }
}
