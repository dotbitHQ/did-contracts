#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use super::constants::CharSetType;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountChar {
    pub char_set_type: CharSetType,
    pub char: String,
}

pub type AccountChars = Vec<AccountChar>;
