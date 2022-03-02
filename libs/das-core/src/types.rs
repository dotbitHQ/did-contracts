use super::{constants::ScriptHashType, error::Error, warn};
use alloc::vec::Vec;
use das_types::{
    constants::{CharSetType, DataType},
    packed::*,
};

macro_rules! config_getter {
    ( $property:ident, $config_type:ty, $config_name:expr ) => {
        pub fn $property(&self) -> Result<$config_type, Error> {
            let reader = self
                .$property
                .as_ref()
                .map(|item| item.as_reader())
                .ok_or_else(|| {
                    warn!(
                        "Can not load {:?}, you need use WitnessesParser::parse_config to parse it first.",
                        $config_name
                    );
                    Error::ConfigIsPartialMissing
                })?;
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

#[derive(Debug)]
pub struct LockScriptTypeIdTable {
    pub always_success: Script,
    pub das_lock: Script,
    pub secp256k1_blake160_signhash_all: Script,
    pub secp256k1_blake160_multisig_all: Script,
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
    pub secondary_market: Option<ConfigCellSecondaryMarket>,
    pub reverse_resolution: Option<ConfigCellReverseResolution>,
    pub sub_account: Option<ConfigCellSubAccount>,
    pub record_key_namespace: Option<Vec<u8>>,
    pub preserved_account: Option<Vec<u8>>,
    pub unavailable_account: Option<Vec<u8>>,
}

impl Configs {
    pub fn new() -> Self {
        Configs::default()
    }

    config_getter!(account, ConfigCellAccountReader, DataType::ConfigCellAccount);
    config_getter!(apply, ConfigCellApplyReader, DataType::ConfigCellApply);
    config_getter!(income, ConfigCellIncomeReader, DataType::ConfigCellIncome);
    config_getter!(main, ConfigCellMainReader, DataType::ConfigCellMain);
    config_getter!(price, ConfigCellPriceReader, DataType::ConfigCellPrice);
    config_getter!(proposal, ConfigCellProposalReader, DataType::ConfigCellProposal);
    config_getter!(profit_rate, ConfigCellProfitRateReader, DataType::ConfigCellProfitRate);
    config_getter!(release, ConfigCellReleaseReader, DataType::ConfigCellRelease);
    config_getter!(
        secondary_market,
        ConfigCellSecondaryMarketReader,
        "ConfigCellSecondaryMarketReader"
    );
    config_getter!(
        reverse_resolution,
        ConfigCellReverseResolutionReader,
        "ConfigCellReverseResolutionReader"
    );
    config_getter!(sub_account, ConfigCellSubAccountReader, "ConfigCellSubAccountReader");

    pub fn record_key_namespace(&self) -> Result<&Vec<u8>, Error> {
        let reader = self.record_key_namespace.as_ref().map(|item| item).ok_or_else(|| {
            warn!(
                "Can not load {:?}, you need use WitnessesParser::parse_config to parse it first.",
                DataType::ConfigCellRecordKeyNamespace
            );
            Error::ConfigIsPartialMissing
        })?;
        Ok(reader)
    }

    pub fn preserved_account(&self) -> Result<&[u8], Error> {
        let reader = self.preserved_account.as_ref().ok_or_else(|| {
            warn!(
                "Can not load {}, you need use WitnessesParser::parse_config to parse it first.",
                "ConfigCellPreservedAccountXX"
            );
            Error::ConfigIsPartialMissing
        })?;
        Ok(reader)
    }

    pub fn unavailable_account(&self) -> Result<&[u8], Error> {
        let reader = self.unavailable_account.as_ref().ok_or_else(|| {
            warn!(
                "Can not load {:?}, you need use WitnessesParser::parse_config to parse it first.",
                DataType::ConfigCellUnAvailableAccount
            );
            Error::ConfigIsPartialMissing
        })?;
        Ok(reader)
    }

    pub fn char_set(&self) -> Result<&Vec<Option<CharSet>>, Error> {
        let reader = self.char_set.as_ref().map(|item| item).ok_or_else(|| {
            warn!(
                "Can not load {}, you need use WitnessesParser::parse_config to parse it first.",
                "ConfigCellCharSetXX"
            );
            Error::ConfigIsPartialMissing
        })?;
        Ok(reader)
    }
}
