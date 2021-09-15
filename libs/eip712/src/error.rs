#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum EIP712EncodingError {
    UndefinedEthABIType = 1,
    InvalidEthABIType = 2,
    UndefinedEIP712Type = 3,
    TypeOfValueIsInvalid = 4,
    FailedWhenEncodingTypes = 5,
    FailedWhenEncodingMessage = 6,
    HexDecodingError = 10,
}
