use super::constants::ScriptHashType;
use super::error::Error;
use alloc::vec::Vec;
use das_types::constants::CharSetType;
use das_types::packed::*;

macro_rules! config_getter {
    ( $property:ident, $config_type:ty ) => {
        pub fn $property(&self) -> Result<$config_type, Error> {
            let reader = self
                .$property
                .as_ref()
                .map(|item| item.as_reader())
                .ok_or(Error::ConfigIsPartialMissing)?;
            Ok(reader)
        }
    };
}

#[derive(Debug)]
pub struct ScriptLiteral {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub args: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CharSet {
    pub name: CharSetType,
    pub global: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct Configs {
    pub account: Option<ConfigCellAccount>,
    pub apply: Option<ConfigCellApply>,
    pub char_set: Option<Vec<Option<CharSet>>>,
    pub income: Option<ConfigCellIncome>,
    pub main: Option<ConfigCellMain>,
    pub price: Option<ConfigCellPrice>,
    pub proposal: Option<ConfigCellProposal>,
    pub profit_rate: Option<ConfigCellProfitRate>,
    pub release: Option<ConfigCellRelease>,
    pub record_key_namespace: Option<Vec<u8>>,
    pub preserved_account: Option<Vec<u8>>,
}

impl Configs {
    pub fn new() -> Self {
        Configs::default()
    }

    config_getter!(account, ConfigCellAccountReader);
    config_getter!(apply, ConfigCellApplyReader);
    config_getter!(income, ConfigCellIncomeReader);
    config_getter!(main, ConfigCellMainReader);
    config_getter!(price, ConfigCellPriceReader);
    config_getter!(proposal, ConfigCellProposalReader);
    config_getter!(profit_rate, ConfigCellProfitRateReader);
    config_getter!(release, ConfigCellReleaseReader);

    pub fn record_key_namespace(&self) -> Result<&Vec<u8>, Error> {
        let reader = self
            .record_key_namespace
            .as_ref()
            .map(|item| item)
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }

    pub fn preserved_account(&self) -> Result<&[u8], Error> {
        let reader = self
            .preserved_account
            .as_ref()
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }

    pub fn char_set(&self) -> Result<&Vec<Option<CharSet>>, Error> {
        let reader = self
            .char_set
            .as_ref()
            .map(|item| item)
            .ok_or(Error::ConfigIsPartialMissing)?;
        Ok(reader)
    }
}
