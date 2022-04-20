use core::convert::TryFrom;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum DasLockType {
    CKBSingle,
    CKBMulti,
    XXX,
    ETH,
    TRON,
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
            x if x == DasLockType::TRON as u8 => Ok(DasLockType::TRON),
            x if x == DasLockType::ETHTypedData as u8 => Ok(DasLockType::ETHTypedData),
            x if x == DasLockType::MIXIN as u8 => Ok(DasLockType::MIXIN),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "mainnet")]
pub const ETH_LIB_CODE_HASH: [u8; 32] = [
    184, 112, 42, 157, 136, 93, 85, 232, 246, 244, 116, 198, 101, 0, 175, 16, 170, 14, 254, 155, 121, 55, 246, 120, 95,
    130, 7, 63, 200, 42, 60, 11,
];

#[cfg(not(feature = "mainnet"))]
pub const ETH_LIB_CODE_HASH: [u8; 32] = [
    114, 136, 18, 7, 241, 131, 151, 251, 114, 137, 71, 94, 28, 208, 216, 64, 104, 55, 4, 5, 126, 140, 166, 6, 43, 114,
    139, 209, 174, 122, 155, 68,
];

#[cfg(feature = "mainnet")]
pub const TRON_LIB_CODE_HASH: [u8; 32] = [
    184, 112, 42, 157, 136, 93, 85, 232, 246, 244, 116, 198, 101, 0, 175, 16, 170, 14, 254, 155, 121, 55, 246, 120, 95,
    130, 7, 63, 200, 42, 60, 11,
];

#[cfg(not(feature = "mainnet"))]
pub const TRON_LIB_CODE_HASH: [u8; 32] = [
    77,158,173,37,107,104,190,10,210,155,235,159,227,120,157,62,225,193,26,104,28,192,50,29,1,118,45,246,222,214,29,43
];
