use core::fmt::Display;

use das_types::packed::DasLockTypeIdTableReader;

pub type DynLibSize = [u8; 192 * 1024];

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DynLibName {
    CKBSignhash,
    CKBMultisig,
    ED25519,
    ETH,
    TRON,
    DOGE,
    WebAuthn
}

impl DynLibName {
    pub fn get_code_hash<'a>(&self, type_id_table_reader: DasLockTypeIdTableReader<'a>) -> &'a [u8] {
        match &self {
            &DynLibName::CKBSignhash => type_id_table_reader.ckb_signhash().raw_data(),
            &DynLibName::CKBMultisig => type_id_table_reader.ckb_multisig().raw_data(),
            &DynLibName::ED25519 => type_id_table_reader.ed25519().raw_data(),
            &DynLibName::ETH => type_id_table_reader.eth().raw_data(),
            &DynLibName::TRON => type_id_table_reader.tron().raw_data(),
            &DynLibName::DOGE => type_id_table_reader.doge().raw_data(),
            &DynLibName::WebAuthn => type_id_table_reader.web_authn().raw_data(),
        }
    }
}

impl Display for DynLibName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
