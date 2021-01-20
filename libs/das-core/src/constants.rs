#[derive(Debug)]
#[repr(u8)]
pub enum ScriptHashType {
    Data,
    Type,
}

#[derive(Debug)]
pub enum ScriptType {
    Lock,
    Type,
}

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";
