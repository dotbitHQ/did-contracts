use das_types::constants::{SystemStatus, TypeScript};
use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum FieldKey {
    SystemStatus,
    AccountCellTypeArgs,
    AccountSaleCellTypeArgs,
    AlwaysSuccessTypeArgs,
    ApplyRegisterCellTypeArgs,
    BalanceCellTypeArgs,
    ConfigCellTypeArgs,
    DeviceKeyListCellTypeArgs,
    DidCellTypeArgs,
    DpointCellTypeArgs,
    IncomeCellTypeArgs,
    OfferCellTypeArgs,
    PreAccountCellTypeArgs,
    ProposalCellTypeArgs,
    ReverseRecordCellTypeArgs,
    ReverseRecordRootCellTypeArgs,
    SubAccountCellTypeArgs,
    DispatchTypeArgs,
    Eip712LibTypeArgs,
    BtcSignSoTypeArgs,
    CkbMultiSignSoTypeArgs,
    CkbSignSoTypeArgs,
    DogeSignSoTypeArgs,
    Ed25519SignSoTypeArgs,
    EthSignSoTypeArgs,
    TronSignSoTypeArgs,
    WebauthnSignSoTypeArgs,
}

impl From<TypeScript> for FieldKey {
    fn from(value: TypeScript) -> Self {
        match value {
            TypeScript::AccountCellType => FieldKey::AccountCellTypeArgs,
            TypeScript::AccountSaleCellType => FieldKey::AccountSaleCellTypeArgs,
            TypeScript::ApplyRegisterCellType => FieldKey::ApplyRegisterCellTypeArgs,
            TypeScript::BalanceCellType => FieldKey::BalanceCellTypeArgs,
            TypeScript::ConfigCellType => FieldKey::ConfigCellTypeArgs,
            TypeScript::DeviceKeyListCellType => FieldKey::DeviceKeyListCellTypeArgs,
            TypeScript::DidCellType => FieldKey::DidCellTypeArgs,
            TypeScript::DPointCellType => FieldKey::DpointCellTypeArgs,
            TypeScript::IncomeCellType => FieldKey::IncomeCellTypeArgs,
            TypeScript::OfferCellType => FieldKey::OfferCellTypeArgs,
            TypeScript::PreAccountCellType => FieldKey::PreAccountCellTypeArgs,
            TypeScript::ProposalCellType => FieldKey::ProposalCellTypeArgs,
            TypeScript::ReverseRecordCellType => FieldKey::ReverseRecordCellTypeArgs,
            TypeScript::ReverseRecordRootCellType => FieldKey::ReverseRecordRootCellTypeArgs,
            TypeScript::SubAccountCellType => FieldKey::SubAccountCellTypeArgs,
            TypeScript::EIP712Lib => FieldKey::Eip712LibTypeArgs,
        }
    }
}

pub enum FieldValue {
    SystemStatus(SystemStatus),
    Hash([u8; 32]),
}
