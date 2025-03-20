#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::{DataType, ACCOUNT_ID_LENGTH};

use crate::error::ConfigError;
use crate::traits::SerializableConfig;
use crate::util;

macro_rules! define_raw_config {
    ( $name:ident, $data_type:expr ) => {
        #[derive(Debug, Default, Clone)]
        pub struct $name {
            pub version: u8,
            pub value: Vec<u8>,
            pub inited: bool,
        }

        impl $name {
            const NAME: &'static str = stringify!($name);
            const DATA_TYPE: DataType = $data_type;

            pub fn new(version: u8, value: &[u8]) -> Result<Self, ConfigError> {
                Ok(Self {
                    version,
                    value: value.to_vec(),
                    inited: true,
                })
            }

            pub fn name(&self) -> &'static str {
                Self::NAME
            }

            pub fn data_type(&self) -> DataType {
                Self::DATA_TYPE
            }

            pub fn version(&self) -> u8 {
                self.version
            }

            pub fn value(&self) -> &[u8] {
                &self.value
            }
        }

        impl SerializableConfig for $name {
            fn from_slice(data: &[u8]) -> Result<Self, ConfigError> {
                let version = data[0];
                let value = (&data[1..]);

                Ok(Self::new(version, &value)?)
            }

            fn as_slice(&self) -> Vec<u8> {
                let mut data = vec![self.version];
                data.extend_from_slice(self.value.as_slice());
                data
            }
        }
    };
}

define_raw_config!(ConfigUnavailableAccount, DataType::ConfigCellUnAvailableAccount);
define_raw_config!(ConfigRecordKeyNamespace, DataType::ConfigCellRecordKeyNamespace);

impl ConfigUnavailableAccount {
    #[cfg(feature = "std")]
    pub fn from_hash_vec(hashes: Vec<String>) -> Result<Self, ConfigError> {
        let mut account_hashes = Vec::new();
        for hash in hashes {
            let account_hash: Vec<u8> = hex::decode(hash.trim_start_matches("0x")).unwrap();
            account_hashes.push((&account_hash[..ACCOUNT_ID_LENGTH]).to_vec());
        }

        account_hashes.sort();
        let raw = account_hashes.into_iter().flatten().collect::<Vec<u8>>();

        Self::new(0, &raw)
    }

    pub fn hash_account(account_without_suffix: &[u8]) -> [u8; ACCOUNT_ID_LENGTH] {
        util::hash_account(account_without_suffix)
    }

    pub fn is_account_exist(&self, account: &[u8]) -> bool {
        let account_hash = util::hash_account(account);
        util::is_account_in_collection(&account_hash, &self.value)
    }
}

impl ConfigRecordKeyNamespace {
    #[cfg(feature = "std")]
    pub fn from_string_vec(mut keys: Vec<String>) -> Result<Self, ConfigError> {
        keys.sort();

        // Join all record keys with 0x00 byte as entity.
        let mut raw = Vec::new();
        for key in keys {
            raw.extend(key.as_bytes());
            raw.extend(&[0u8]);
        }

        Self::new(0, &raw)
    }

    pub fn to_key_list(&self) -> Vec<&[u8]> {
        // extract all the keys, which are split by 0x00
        let mut key_start_at = 0;
        let mut key_list = Vec::new();
        for (index, item) in self.value.iter().enumerate() {
            if *item == 0 {
                let key_vec = &self.value[key_start_at..index];
                key_start_at = index + 1;

                key_list.push(key_vec);
            }
        }

        key_list
    }
}
