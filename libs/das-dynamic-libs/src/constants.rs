use core::convert::TryFrom;

// TODO This is copy from das-core/src/constants, it should be unified as soon as possible.
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum DasLockType {
    CKBSingle,
    CKBMulti,
    XXX,
    ETH,
    TRX,
    ETHTypedData,
    MIXIN,
}

impl TryFrom<u8> for DasLockType {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == DasLockType::CKBSingle as u8 => Ok(DasLockType::CKBSingle),
            x if x == DasLockType::CKBMulti as u8 => Ok(DasLockType::CKBMulti),
            x if x == DasLockType::XXX as u8 => Ok(DasLockType::XXX),
            x if x == DasLockType::ETH as u8 => Ok(DasLockType::ETH),
            x if x == DasLockType::TRX as u8 => Ok(DasLockType::TRX),
            x if x == DasLockType::ETHTypedData as u8 => Ok(DasLockType::ETHTypedData),
            x if x == DasLockType::MIXIN as u8 => Ok(DasLockType::MIXIN),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "mainnet")]
pub const ETH_LIB_CODE_HASH: [u8; 32] = [
    114, 136, 18, 7, 241, 131, 151, 251, 114, 137, 71, 94, 28, 208, 216, 64, 104, 55, 4, 5, 126, 140, 166, 6, 43, 114,
    139, 209, 174, 122, 155, 68,
];

#[cfg(not(feature = "mainnet"))]
pub const ETH_LIB_CODE_HASH: [u8; 32] = [
    114, 136, 18, 7, 241, 131, 151, 251, 114, 137, 71, 94, 28, 208, 216, 64, 104, 55, 4, 5, 126, 140, 166, 6, 43, 114,
    139, 209, 174, 122, 155, 68,
];
