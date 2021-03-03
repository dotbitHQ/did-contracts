use super::constants::ScriptHashType;
use super::error::Error;
use alloc::vec::Vec;
use das_types::packed::*;

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

    pub fn main(&self) -> Result<ConfigCellMainReader, Error> {
        let reader = self
            .main
            .as_ref()
            .map(|item| item.as_reader())
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }

    pub fn register(&self) -> Result<ConfigCellRegisterReader, Error> {
        let reader = self
            .register
            .as_ref()
            .map(|item| item.as_reader())
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }

    pub fn bloom_filter(&self) -> Result<&[u8], Error> {
        let reader = self
            .bloom_filter
            .as_ref()
            .map(|item| item.as_slice())
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }

    pub fn market(&self) -> Result<ConfigCellMarketReader, Error> {
        let reader = self
            .market
            .as_ref()
            .map(|item| item.as_reader())
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }
}
