#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::{CharSetType, DataType};

use crate::error::ConfigError;
use crate::traits::SerializableConfig;
use crate::util;

#[derive(Debug, Clone)]
pub struct ConfigCharSet {
    pub data_type: DataType,
    pub version: u8,
    pub inited: bool,

    pub char_set_type: CharSetType,
    pub global: bool,
    pub value: Vec<u8>,
}

impl Default for ConfigCharSet {
    fn default() -> Self {
        ConfigCharSet {
            data_type: DataType::ConfigCellCharSetEmoji,
            version: 1,
            inited: false,
            char_set_type: CharSetType::Emoji,
            global: false,
            value: Vec::new(),
        }
    }
}

impl ConfigCharSet {
    const NAME: &'static str = "ConfigCharSet";
    const DEFAULT_DATA_TYPE: DataType = DataType::ConfigCellCharSetEmoji;
    const DEFAULT_CHAR_SET_TYPE: CharSetType = CharSetType::Emoji;

    pub fn char_set_type_to_data_type(char_set: CharSetType) -> DataType {
        DataType::try_from(char_set as u32 + 100000).unwrap()
    }

    #[cfg(feature = "std")]
    pub fn from_string_vec(
        data_type: DataType,
        char_set_type: CharSetType,
        is_global: bool,
        mut chars: Vec<String>,
    ) -> Result<Self, ConfigError> {
        chars.sort();

        // Join all record keys with 0x00 byte as entity.
        let mut raw = Vec::new();
        for key in chars {
            raw.extend(key.as_bytes());
            raw.extend(&[0u8]);
        }

        Self::new(data_type, 0, char_set_type, is_global, &raw)
    }

    pub fn new(
        data_type: DataType,
        version: u8,
        char_set_type: CharSetType,
        is_global: bool,
        value: &[u8],
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            data_type,
            version,
            inited: true,
            char_set_type,
            global: is_global,
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

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type
    }

    pub fn set_char_set_type(&mut self, char_set_type: CharSetType) {
        self.char_set_type = char_set_type
    }

    pub fn is_account_exist(&self, account_id: &[u8]) -> bool {
        util::is_account_in_collection(account_id, &self.value)
    }
}

impl SerializableConfig for ConfigCharSet {
    fn from_slice(data: &[u8]) -> Result<Self, ConfigError> {
        let version = data[0];
        let is_global = data[1] == 1u8;
        let value = &data[2..];

        Ok(Self::new(
            Self::DEFAULT_DATA_TYPE,
            version,
            Self::DEFAULT_CHAR_SET_TYPE,
            is_global,
            value,
        )
        .unwrap())
    }

    fn as_slice(&self) -> Vec<u8> {
        let is_global = if self.global { 1u8 } else { 0u8 };
        let mut data = vec![self.version, is_global];
        data.extend_from_slice(self.value.as_slice());
        data
    }
}
