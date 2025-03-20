use core::fmt;

#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::DataType;

use crate::error::ConfigError;
use crate::traits::SerializableConfig;

#[derive(Default, Clone)]
pub struct ConfigSMTNodeWhiteList {
    pub version: u8,
    pub value: Vec<[u8; 32]>,
    pub inited: bool,
}

impl fmt::Debug for ConfigSMTNodeWhiteList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "  version: {},", self.version)?;
        writeln!(f, "  value: [")?;
        for array in &self.value {
            write!(f, "    [{}],", hex::encode(array))?;
        }
        writeln!(f, "  ],")?;
        writeln!(f, "  inited: {}", self.inited)?;
        writeln!(f, "}}")
    }
}

impl ConfigSMTNodeWhiteList {
    const NAME: &'static str = "ConfigPreservedAccount";
    const DATA_TYPE: DataType = DataType::ConfigCellSMTNodeWhitelist;

    #[cfg(feature = "std")]
    pub fn from_hash_vec(hash_hex_list: Vec<String>) -> Result<Self, ConfigError> {
        let mut hash_bytes_list = Vec::new();
        for hash in hash_hex_list {
            let hash_bytes: Vec<u8> = hex::decode(hash.trim_start_matches("0x")).unwrap();
            hash_bytes_list.push(hash_bytes);
        }

        hash_bytes_list.sort();

        let raw = hash_bytes_list.into_iter().flatten().collect::<Vec<u8>>();
        Self::new(0, &raw)
    }

    pub fn new(version: u8, value: &[u8]) -> Result<Self, ConfigError> {
        let mut data = vec![];
        let mut from: usize = 0;
        let mut to: usize = 32;
        loop {
            match value.get(from..to) {
                Some(val) => {
                    let mut tmp = [0u8; 32];
                    tmp.copy_from_slice(val);
                    data.push(tmp);

                    from = to;
                    to += 32;
                }
                None => break,
            }
        }

        Ok(Self {
            version,
            value: data,
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

    pub fn value(&self) -> &[[u8; 32]] {
        &self.value
    }
}

impl SerializableConfig for ConfigSMTNodeWhiteList {
    fn from_slice(data: &[u8]) -> Result<Self, ConfigError> {
        let version = data[0];
        let value = &data[1..];

        Ok(Self::new(version, value).unwrap())
    }

    fn as_slice(&self) -> Vec<u8> {
        let mut data = vec![self.version];
        for val in self.value.iter() {
            data.extend_from_slice(val);
        }

        data
    }
}
