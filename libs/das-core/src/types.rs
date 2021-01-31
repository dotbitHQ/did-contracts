use super::constants::ScriptHashType;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct ScriptLiteral {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub args: Vec<u8>,
}
