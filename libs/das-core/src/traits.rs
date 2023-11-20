use das_types::constants::DataType;
use das_types::packed::DataEntity;
use das_types::prelude::Entity;

use crate::util;

pub trait Blake2BHash {
    fn blake2b_256(&self) -> [u8; 32];
}

impl<T: Entity> Blake2BHash for T {
    default fn blake2b_256(&self) -> [u8; 32] {
        util::blake2b_256(self.as_slice())
    }
}

impl Blake2BHash for DataEntity {
    fn blake2b_256(&self) -> [u8; 32] {
        util::blake2b_256(self.entity().as_reader().raw_data())
    }
}

pub trait TryFromBytes<T> {
    fn try_from_bytes(value: T) -> molecule::error::VerificationResult<Self>
    where
        Self: Sized;
}

impl<A> TryFromBytes<molecule::bytes::Bytes> for A
where
    A: Entity + Sized,
{
    fn try_from_bytes(value: molecule::bytes::Bytes) -> molecule::error::VerificationResult<Self> {
        Self::from_compatible_slice(&value)
    }
}

impl<A> TryFromBytes<das_types::packed::Bytes> for A
where
    A: Entity + Sized,
{
    fn try_from_bytes(value: das_types::packed::Bytes) -> molecule::error::VerificationResult<Self> {
        debug!("value: {:?}", value);
        Self::from_compatible_slice(&value.as_bytes())
    }
}

impl<A> TryFromBytes<ckb_std::ckb_types::packed::Bytes> for A
where
    A: Entity + Sized,
{
    fn try_from_bytes(value: ckb_std::ckb_types::packed::Bytes) -> molecule::error::VerificationResult<Self> {
        Self::from_compatible_slice(&value.as_bytes())
    }
}

pub trait GetDataType {
    fn get_type_constant() -> DataType;
}

// impl<T, H> GetDataType for EntityWrapper<T, H> where T: Entity {
//     fn get_type_constant() -> DataType {
//         match T::NAME {
//             "DeviceKeyListCellData" => DataType::DeviceKeyListEntityData,
//             _ => unreachable!()
//         }
//     }
// }

impl<T> GetDataType for T
where
    T: Entity,
{
    default fn get_type_constant() -> DataType {
        match T::NAME {
            "ActionData" => DataType::ActionData,
            "AccountCellData" => DataType::AccountCellData,
            "AccountSaleCellData" => DataType::AccountSaleCellData,
            "AccountAuctionCellData" => DataType::AccountAuctionCellData,
            "ProposalCellData" => DataType::ProposalCellData,
            "PreAccountCellData" => DataType::PreAccountCellData,
            "IncomeCellData" => DataType::IncomeCellData,
            "OfferCellData" => DataType::OfferCellData,
            "SubAccount" => DataType::SubAccount,
            "SubAccountMintSign" => DataType::SubAccountMintSign,
            "ReverseRecord" => DataType::ReverseRecord,
            "SubAccountPriceRule" => DataType::SubAccountPriceRule,
            "SubAccountPreservedRule" => DataType::SubAccountPreservedRule,
            // "DeviceKeyListEntityData" => DataType::DeviceKeyListEntityData,
            "SubAccountRenewSign" => DataType::SubAccountRenewSign,
            "DeviceKeyListCellData" => DataType::DeviceKeyListCellData,
            "ConfigCellAccount" => DataType::ConfigCellAccount,
            "ConfigCellApply" => DataType::ConfigCellApply,
            "ConfigCellIncome" => DataType::ConfigCellIncome,
            "ConfigCellMain" => DataType::ConfigCellMain,
            "ConfigCellPrice" => DataType::ConfigCellPrice,
            "ConfigCellProposal" => DataType::ConfigCellProposal,
            "ConfigCellProfitRate" => DataType::ConfigCellProfitRate,
            "ConfigCellRecordKeyNamespace" => DataType::ConfigCellRecordKeyNamespace,
            "ConfigCellRelease" => DataType::ConfigCellRelease,
            "ConfigCellUnAvailableAccount" => DataType::ConfigCellUnAvailableAccount,
            "ConfigCellSecondaryMarket" => DataType::ConfigCellSecondaryMarket,
            "ConfigCellReverseResolution" => DataType::ConfigCellReverseResolution,
            "ConfigCellSubAccount" => DataType::ConfigCellSubAccount,
            "ConfigCellSubAccountBetaList" => DataType::ConfigCellSubAccountBetaList,
            "ConfigCellSystemStatus" => DataType::ConfigCellSystemStatus,
            "ConfigCellSMTNodeWhitelist" => DataType::ConfigCellSMTNodeWhitelist,
            "ConfigCellPreservedAccount00" => DataType::ConfigCellPreservedAccount00,
            "ConfigCellPreservedAccount01" => DataType::ConfigCellPreservedAccount01,
            "ConfigCellPreservedAccount02" => DataType::ConfigCellPreservedAccount02,
            "ConfigCellPreservedAccount03" => DataType::ConfigCellPreservedAccount03,
            "ConfigCellPreservedAccount04" => DataType::ConfigCellPreservedAccount04,
            "ConfigCellPreservedAccount05" => DataType::ConfigCellPreservedAccount05,
            "ConfigCellPreservedAccount06" => DataType::ConfigCellPreservedAccount06,
            "ConfigCellPreservedAccount07" => DataType::ConfigCellPreservedAccount07,
            "ConfigCellPreservedAccount08" => DataType::ConfigCellPreservedAccount08,
            "ConfigCellPreservedAccount09" => DataType::ConfigCellPreservedAccount09,
            "ConfigCellPreservedAccount10" => DataType::ConfigCellPreservedAccount10,
            "ConfigCellPreservedAccount11" => DataType::ConfigCellPreservedAccount11,
            "ConfigCellPreservedAccount12" => DataType::ConfigCellPreservedAccount12,
            "ConfigCellPreservedAccount13" => DataType::ConfigCellPreservedAccount13,
            "ConfigCellPreservedAccount14" => DataType::ConfigCellPreservedAccount14,
            "ConfigCellPreservedAccount15" => DataType::ConfigCellPreservedAccount15,
            "ConfigCellPreservedAccount16" => DataType::ConfigCellPreservedAccount16,
            "ConfigCellPreservedAccount17" => DataType::ConfigCellPreservedAccount17,
            "ConfigCellPreservedAccount18" => DataType::ConfigCellPreservedAccount18,
            "ConfigCellPreservedAccount19" => DataType::ConfigCellPreservedAccount19,
            "ConfigCellCharSetEmoji" => DataType::ConfigCellCharSetEmoji,
            "ConfigCellCharSetDigit" => DataType::ConfigCellCharSetDigit,
            "ConfigCellCharSetEn" => DataType::ConfigCellCharSetEn,
            "ConfigCellCharSetZhHans" => DataType::ConfigCellCharSetZhHans,
            "ConfigCellCharSetZhHant" => DataType::ConfigCellCharSetZhHant,
            "ConfigCellCharSetJa" => DataType::ConfigCellCharSetJa,
            "ConfigCellCharSetKo" => DataType::ConfigCellCharSetKo,
            "ConfigCellCharSetRu" => DataType::ConfigCellCharSetRu,
            "ConfigCellCharSetTr" => DataType::ConfigCellCharSetTr,
            "ConfigCellCharSetTh" => DataType::ConfigCellCharSetTh,
            "ConfigCellCharSetVi" => DataType::ConfigCellCharSetVi,
            "Config" => DataType::ConfigCellMain,
            _ => unreachable!(),
        }
    }
}
