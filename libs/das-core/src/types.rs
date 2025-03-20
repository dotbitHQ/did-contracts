use alloc::vec::Vec;

use das_types::constants::CharSetType;
use das_types::packed::*;

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
