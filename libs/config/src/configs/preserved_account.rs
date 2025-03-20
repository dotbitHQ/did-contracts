#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::{DataType, ACCOUNT_ID_LENGTH, PRESERVED_ACCOUNT_CELL_COUNT};

use crate::error::ConfigError;
use crate::traits::SerializableConfig;
use crate::util;
#[derive(Debug, Clone)]
pub struct ConfigPreservedAccount {
    pub data_type: DataType,
    pub version: u8,
    pub inited: bool,
    pub value: Vec<u8>,
}

impl Default for ConfigPreservedAccount {
    fn default() -> Self {
        ConfigPreservedAccount {
            data_type: ConfigPreservedAccount::DEFAULT_DATA_TYPE,
            version: 0,
            inited: false,
            value: Vec::new(),
        }
    }
}

impl ConfigPreservedAccount {
    const NAME: &'static str = "ConfigPreservedAccount";
    const DATA_TYPE_START_AT: u32 = 10000;
    const DEFAULT_DATA_TYPE: DataType = DataType::ConfigCellPreservedAccount00;

    pub fn get_data_type_of_account(account_without_suffix: &[u8]) -> DataType {
        let account_hash = Self::hash_account(account_without_suffix);
        let index = (account_hash[0] % PRESERVED_ACCOUNT_CELL_COUNT) as usize;
        let data_type = DataType::try_from(index as u32 + Self::DATA_TYPE_START_AT).unwrap();

        data_type
    }

    pub fn get_data_type_of_index(index: usize) -> DataType {
        DataType::try_from(index as u32 + Self::DATA_TYPE_START_AT).unwrap()
    }

    pub fn hash_account(account_without_suffix: &[u8]) -> [u8; ACCOUNT_ID_LENGTH] {
        util::hash_account(account_without_suffix)
    }

    pub fn new(data_type: DataType, version: u8, value: &[u8]) -> Result<Self, ConfigError> {
        Ok(Self {
            data_type,
            version,
            inited: true,
            value: value.to_vec(),
        })
    }

    pub fn name(&self) -> &'static str {
        Self::NAME
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn is_account_exist(&self, account_without_suffix: &[u8]) -> bool {
        let account_hash = Self::hash_account(account_without_suffix);
        util::is_account_in_collection(&account_hash, &self.value)
    }
}

impl SerializableConfig for ConfigPreservedAccount {
    fn from_slice(data: &[u8]) -> Result<Self, ConfigError> {
        let version = data[0];
        let value = &data[1..];

        Ok(Self::new(ConfigPreservedAccount::DEFAULT_DATA_TYPE, version, value).unwrap())
    }

    fn as_slice(&self) -> Vec<u8> {
        let mut data = vec![self.version];
        data.extend_from_slice(self.value.as_slice());
        data
    }
}
