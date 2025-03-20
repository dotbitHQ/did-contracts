pub type DynLibSize = [u8; 192 * 1024];

#[deprecated]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DynLibName {
    CKBSignhash,
    CKBMultisig,
    ED25519,
    ETH,
    TRON,
    DOGE,
    WebAuthn,
    BTC,
}
