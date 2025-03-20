use core::fmt;

#[cfg(feature = "no_std")]
use alloc::format;
#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::{DataType, SystemStatus};
use das_types::packed::{Bytes, BytesVec};
use das_types::util as types_util;
use molecule::prelude::{Builder, Entity};

use crate::constants::FieldKey;
use crate::error::ConfigError;
use crate::traits::SerializableConfig;

#[derive(Debug, Clone)]
pub enum ConfigMainField {
    SystemStatus(SystemStatusField),
    HashField(HashField),
}

impl ConfigMainField {
    fn from_slice(bytes: &[u8]) -> Result<ConfigMainField, ConfigError> {
        let mut key_bytes = [0u8; 4];
        key_bytes.copy_from_slice(&bytes[0..4]);
        let key = match FieldKey::try_from(u32::from_le_bytes(key_bytes)) {
            Ok(key) => key,
            Err(_) => {
                return Err(ConfigError::ConfigCellMainFieldUndefined {
                    key: u32::from_le_bytes(key_bytes),
                })
            }
        };

        let field = match key {
            FieldKey::SystemStatus => ConfigMainField::SystemStatus(SystemStatusField::new(&bytes[4..])?),
            FieldKey::AccountCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "AccountCellTypeArgs",
                FieldKey::AccountCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::AccountSaleCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "AccountSaleCellTypeArgs",
                FieldKey::AccountSaleCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::AlwaysSuccessTypeArgs => ConfigMainField::HashField(HashField::new(
                "AlwaysSuccessTypeArgs",
                FieldKey::AlwaysSuccessTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::ApplyRegisterCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "ApplyRegisterCellTypeArgs",
                FieldKey::ApplyRegisterCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::BalanceCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "BalanceCellTypeArgs",
                FieldKey::BalanceCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::ConfigCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "ConfigCellTypeArgs",
                FieldKey::ConfigCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::DeviceKeyListCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "DeviceKeyListCellTypeArgs",
                FieldKey::DeviceKeyListCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::DidCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "DidCellTypeArgs",
                FieldKey::DidCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::DpointCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "DpointCellTypeArgs",
                FieldKey::DpointCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::IncomeCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "IncomeCellTypeArgs",
                FieldKey::IncomeCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::OfferCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "OfferCellTypeArgs",
                FieldKey::OfferCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::PreAccountCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "PreAccountCellTypeArgs",
                FieldKey::PreAccountCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::ProposalCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "ProposalCellTypeArgs",
                FieldKey::ProposalCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::ReverseRecordCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "ReverseRecordCellTypeArgs",
                FieldKey::ReverseRecordCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::ReverseRecordRootCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "ReverseRecordRootCellTypeArgs",
                FieldKey::ReverseRecordRootCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::SubAccountCellTypeArgs => ConfigMainField::HashField(HashField::new(
                "SubAccountCellTypeArgs",
                FieldKey::SubAccountCellTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::DispatchTypeArgs => ConfigMainField::HashField(HashField::new(
                "DispatchTypeArgs",
                FieldKey::DispatchTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::Eip712LibTypeArgs => ConfigMainField::HashField(HashField::new(
                "Eip712LibTypeArgs",
                FieldKey::Eip712LibTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::BtcSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "BtcSignSoTypeArgs",
                FieldKey::BtcSignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::CkbMultiSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "CkbMultiSignSoTypeArgs",
                FieldKey::CkbMultiSignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::CkbSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "CkbSignSoTypeArgs",
                FieldKey::CkbSignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::DogeSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "DogeSignSoTypeArgs",
                FieldKey::DogeSignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::Ed25519SignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "Ed25519SignSoTypeArgs",
                FieldKey::Ed25519SignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::EthSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "EthSignSoTypeArgs",
                FieldKey::EthSignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::TronSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "TronSignSoTypeArgs",
                FieldKey::TronSignSoTypeArgs,
                &bytes[4..],
            )?),
            FieldKey::WebauthnSignSoTypeArgs => ConfigMainField::HashField(HashField::new(
                "WebauthnSignSoTypeArgs",
                FieldKey::WebauthnSignSoTypeArgs,
                &bytes[4..],
            )?),
        };

        Ok(field)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            ConfigMainField::SystemStatus(field) => {
                let mut bytes = vec![];
                let key_bytes = (field.key as u32).to_le_bytes();
                bytes.extend_from_slice(&key_bytes);
                bytes.extend_from_slice(&[field.value as u8]);
                bytes
            }
            ConfigMainField::HashField(field) => {
                let mut bytes = vec![];
                let key_bytes = (field.key as u32).to_le_bytes();
                bytes.extend_from_slice(&key_bytes);
                bytes.extend_from_slice(&field.value);
                bytes
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStatusField {
    pub name: &'static str,
    pub key: FieldKey,
    pub value: SystemStatus,
}

impl SystemStatusField {
    fn new(data: &[u8]) -> Result<Self, ConfigError> {
        let status = if data[0] == 0 {
            SystemStatus::Off
        } else if data[0] == 1 {
            SystemStatus::On
        } else {
            return Err(ConfigError::ConfigCellMainFieldDecodingError { key: "SystemStatus" });
        };

        Ok(Self {
            name: "SystemStatus",
            key: FieldKey::SystemStatus,
            value: status,
        })
    }
}

#[derive(Clone)]
pub struct HashField {
    pub name: &'static str,
    pub key: FieldKey,
    pub value: [u8; 32],
}

impl HashField {
    fn new(name: &'static str, key: FieldKey, data: &[u8]) -> Result<Self, ConfigError> {
        if data.len() != 32 {
            return Err(ConfigError::ConfigCellMainFieldDecodingError { key: stringify!($name) });
        }

        let mut value = [0u8; 32];
        value.copy_from_slice(&data[0..32]);

        Ok(Self { name, key, value })
    }
}

impl fmt::Debug for HashField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashField")
            .field("name", &self.name)
            .field("key", &self.key)
            .finish()?;

        write!(f, "\n  value: {}\n", hex::encode(self.value))
    }
}

#[derive(Debug, Default)]
pub struct ConfigMain {
    pub version: u8,
    pub inited: bool,
    pub value: Vec<ConfigMainField>,
}

impl ConfigMain {
    const NAME: &'static str = "ConfigMain";
    const DATA_TYPE: DataType = DataType::ConfigCellMain;

    #[cfg(feature = "std")]
    pub fn from_fields(fields: Vec<(FieldKey, Vec<u8>)>) -> Result<Self, ConfigError> {
        let mut bytes_vec_builder = BytesVec::new_builder();
        for (key, value) in fields {
            let mut bytes = vec![];
            let key_bytes = (key as u32).to_le_bytes();
            bytes.extend_from_slice(&key_bytes);
            bytes.extend_from_slice(&value);

            let item = Bytes::from(bytes);
            bytes_vec_builder = bytes_vec_builder.push(item);
        }
        let bytes_vec = bytes_vec_builder.build();

        Self::new(0, &bytes_vec.as_slice())
    }

    pub fn new(version: u8, value: &[u8]) -> Result<Self, ConfigError> {
        let entity =
            BytesVec::from_compatible_slice(value).map_err(|_| ConfigError::DecodingError { name: Self::NAME })?;

        let mut fields = vec![];
        for item in entity.as_reader().iter() {
            let field = match ConfigMainField::from_slice(item.raw_data()) {
                Ok(field) => field,
                // Unkown key should be safely ignored for compatibility.
                Err(ConfigError::ConfigCellMainFieldUndefined { .. }) => continue,
                Err(err) => return Err(err),
            };
            fields.push(field);
        }

        Ok(Self {
            version,
            inited: true,
            value: fields,
        })
    }

    pub fn data_type(&self) -> DataType {
        Self::DATA_TYPE
    }

    pub fn get(&self, key: &FieldKey) -> Option<&ConfigMainField> {
        self.value.iter().find(|field| match field {
            ConfigMainField::SystemStatus(field) => &field.key == key,
            ConfigMainField::HashField(field) => &field.key == key,
        })
    }

    pub fn status(&self) -> SystemStatus {
        let field = self.get(&FieldKey::SystemStatus).unwrap();
        match field {
            ConfigMainField::SystemStatus(field) => field.value,
            _ => unreachable!(),
        }
    }

    pub fn get_type_id_of(&self, key: FieldKey) -> Result<[u8; 32], ConfigError> {
        let type_args = match self.get(&key) {
            Some(ConfigMainField::HashField(field)) => field.value,
            _ => {
                return Err(ConfigError::ConfigCellMainFieldMissing {
                    expected_key: format!("{:?}", key),
                })
            }
        };

        Ok(types_util::to_type_id(&type_args))
    }
}

impl SerializableConfig for ConfigMain {
    fn from_slice(data: &[u8]) -> Result<Self, ConfigError> {
        let version = data[0];
        let value = &data[1..];

        Self::new(version, value)
    }

    fn as_slice(&self) -> Vec<u8> {
        let mut bytes_vec_builder = BytesVec::new_builder();
        for field in self.value.iter() {
            let item = Bytes::from(field.to_vec());
            bytes_vec_builder = bytes_vec_builder.push(item);
        }
        let bytes_vec = bytes_vec_builder.build();

        let mut data = vec![self.version];
        data.extend_from_slice(bytes_vec.as_slice());
        data
    }
}
