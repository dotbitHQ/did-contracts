#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use crate::error::ConfigError;

pub trait SerializableConfig {
    fn from_slice(data: &[u8]) -> Result<Self, ConfigError>
    where
        Self: Sized;
    fn as_slice(&self) -> Vec<u8>;
}
