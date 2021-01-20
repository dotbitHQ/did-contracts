use super::constants::ScriptHashType;

#[derive(Debug)]
pub struct ScriptLiteral {
    pub code_hash: &'static str,
    pub hash_type: ScriptHashType,
    pub args: &'static str,
}
