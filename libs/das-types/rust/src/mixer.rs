#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::string::ToString;
#[cfg(feature = "no_std")]
use alloc::string::String;

use molecule::error::{VerificationError, VerificationResult};
use molecule::prelude::Entity;

use super::prettier::Prettier;
use super::schemas::packed::*;

macro_rules! gen_trait_common_fns {
    ({$( $fn_name:ident -> $fn_return:ty ),+}) => {
        $(fn $fn_name(&self) -> $fn_return;)+
    };
}

macro_rules! gen_trait_field_fns {
    ({$( $field:ident -> $field_type:ty ),+}) => {
        $(fn $field(&self) -> $field_type;)+
    };
}

macro_rules! gen_impl_field_fns {
    ({$( $field:ident -> $field_type:ty ),+}) => {
        $(
            fn $field(&self) -> $field_type {
                self.$field()
            }
        )+
    };
}

pub trait PreAccountCellDataMixer {
    gen_trait_common_fns!({
        version -> u32,
        as_reader -> Box<dyn PreAccountCellDataReaderMixer + '_>
    });
}

impl PreAccountCellDataMixer for PreAccountCellDataV1 {
    fn version(&self) -> u32 {
        1
    }

    fn as_reader(&self) -> Box<dyn PreAccountCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

impl PreAccountCellDataMixer for PreAccountCellDataV2 {
    fn version(&self) -> u32 {
        2
    }

    fn as_reader(&self) -> Box<dyn PreAccountCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

impl PreAccountCellDataMixer for PreAccountCellData {
    fn version(&self) -> u32 {
        3
    }

    fn as_reader(&self) -> Box<dyn PreAccountCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

pub trait PreAccountCellDataReaderMixer<'r> {
    gen_trait_common_fns!({
        version -> u32,
        try_into_v1 -> VerificationResult<PreAccountCellDataV1Reader<'r>>,
        try_into_v2 -> VerificationResult<PreAccountCellDataV2Reader<'r>>,
        try_into_latest -> VerificationResult<PreAccountCellDataReader<'r>>
    });

    gen_trait_field_fns!({
        account -> AccountCharsReader<'r>,
        refund_lock -> ScriptReader<'r>,
        owner_lock_args -> BytesReader<'r>,
        inviter_id -> BytesReader<'r>,
        inviter_lock -> ScriptOptReader<'r>,
        channel_lock -> ScriptOptReader<'r>,
        price -> PriceConfigReader<'r>,
        quote -> Uint64Reader<'r>,
        invited_discount -> Uint32Reader<'r>,
        created_at -> Uint64Reader<'r>
    });
}

impl<'r> PreAccountCellDataReaderMixer<'r> for PreAccountCellDataV1Reader<'r> {
    fn version(&self) -> u32 {
        1
    }

    fn try_into_v1(&self) -> VerificationResult<PreAccountCellDataV1Reader<'r>> {
        PreAccountCellDataV1Reader::from_slice(self.as_slice())
    }

    fn try_into_v2(&self) -> VerificationResult<PreAccountCellDataV2Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "PreAccountCellDataV1Reader".to_string(),
        ))
    }

    fn try_into_latest(&self) -> VerificationResult<PreAccountCellDataReader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "PreAccountCellDataV1Reader".to_string(),
        ))
    }

    gen_impl_field_fns!({
        account -> AccountCharsReader<'r>,
        refund_lock -> ScriptReader<'r>,
        owner_lock_args -> BytesReader<'r>,
        inviter_id -> BytesReader<'r>,
        inviter_lock -> ScriptOptReader<'r>,
        channel_lock -> ScriptOptReader<'r>,
        price -> PriceConfigReader<'r>,
        quote -> Uint64Reader<'r>,
        invited_discount -> Uint32Reader<'r>,
        created_at -> Uint64Reader<'r>
    });
}

impl<'r> PreAccountCellDataReaderMixer<'r> for PreAccountCellDataV2Reader<'r> {
    fn version(&self) -> u32 {
        2
    }

    fn try_into_v1(&self) -> VerificationResult<PreAccountCellDataV1Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "PreAccountCellDataV2Reader".to_string(),
        ))
    }

    fn try_into_v2(&self) -> VerificationResult<PreAccountCellDataV2Reader<'r>> {
        PreAccountCellDataV2Reader::from_slice(self.as_slice())
    }

    fn try_into_latest(&self) -> VerificationResult<PreAccountCellDataReader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "PreAccountCellDataV2Reader".to_string(),
        ))
    }

    gen_impl_field_fns!({
        account -> AccountCharsReader<'r>,
        refund_lock -> ScriptReader<'r>,
        owner_lock_args -> BytesReader<'r>,
        inviter_id -> BytesReader<'r>,
        inviter_lock -> ScriptOptReader<'r>,
        channel_lock -> ScriptOptReader<'r>,
        price -> PriceConfigReader<'r>,
        quote -> Uint64Reader<'r>,
        invited_discount -> Uint32Reader<'r>,
        created_at -> Uint64Reader<'r>
    });
}

impl<'r> PreAccountCellDataReaderMixer<'r> for PreAccountCellDataReader<'r> {
    fn version(&self) -> u32 {
        3
    }

    fn try_into_v1(&self) -> VerificationResult<PreAccountCellDataV1Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "PreAccountCellDataReader".to_string(),
        ))
    }

    fn try_into_v2(&self) -> VerificationResult<PreAccountCellDataV2Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "PreAccountCellDataReader".to_string(),
        ))
    }

    fn try_into_latest(&self) -> VerificationResult<PreAccountCellDataReader<'r>> {
        PreAccountCellDataReader::from_slice(self.as_slice())
    }

    gen_impl_field_fns!({
        account -> AccountCharsReader<'r>,
        refund_lock -> ScriptReader<'r>,
        owner_lock_args -> BytesReader<'r>,
        inviter_id -> BytesReader<'r>,
        inviter_lock -> ScriptOptReader<'r>,
        channel_lock -> ScriptOptReader<'r>,
        price -> PriceConfigReader<'r>,
        quote -> Uint64Reader<'r>,
        invited_discount -> Uint32Reader<'r>,
        created_at -> Uint64Reader<'r>
    });
}

pub trait AccountCellDataMixer {
    gen_trait_common_fns!({
        version -> u32,
        as_reader -> Box<dyn AccountCellDataReaderMixer + '_>
    });
}

impl AccountCellDataMixer for AccountCellDataV2 {
    fn version(&self) -> u32 {
        2
    }

    fn as_reader(&self) -> Box<dyn AccountCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

impl AccountCellDataMixer for AccountCellDataV3 {
    fn version(&self) -> u32 {
        3
    }

    fn as_reader(&self) -> Box<dyn AccountCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

impl AccountCellDataMixer for AccountCellData {
    fn version(&self) -> u32 {
        4
    }

    fn as_reader(&self) -> Box<dyn AccountCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

pub trait AccountCellDataReaderMixer<'r> {
    gen_trait_common_fns!({
        version -> u32,
        try_into_v2 -> VerificationResult<AccountCellDataV2Reader<'r>>,
        try_into_v3 -> VerificationResult<AccountCellDataV3Reader<'r>>,
        try_into_latest -> VerificationResult<AccountCellDataReader<'r>>
    });

    gen_trait_field_fns!({
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        registered_at -> Uint64Reader<'r>,
        last_transfer_account_at -> Uint64Reader<'r>,
        last_edit_manager_at -> Uint64Reader<'r>,
        last_edit_records_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>
    });
}

impl<'r> AccountCellDataReaderMixer<'r> for AccountCellDataV2Reader<'r> {
    fn version(&self) -> u32 {
        2
    }

    fn try_into_v2(&self) -> VerificationResult<AccountCellDataV2Reader<'r>> {
        AccountCellDataV2Reader::from_slice(self.as_slice())
    }

    fn try_into_v3(&self) -> VerificationResult<AccountCellDataV3Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "AccountCellDataV3Reader".to_string(),
        ))
    }

    fn try_into_latest(&self) -> VerificationResult<AccountCellDataReader<'r>> {
        Err(VerificationError::OffsetsNotMatch("AccountCellDataReader".to_string()))
    }

    gen_impl_field_fns!({
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        registered_at -> Uint64Reader<'r>,
        last_transfer_account_at -> Uint64Reader<'r>,
        last_edit_manager_at -> Uint64Reader<'r>,
        last_edit_records_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>
    });
}

impl<'r> AccountCellDataReaderMixer<'r> for AccountCellDataV3Reader<'r> {
    fn version(&self) -> u32 {
        3
    }

    fn try_into_v2(&self) -> VerificationResult<AccountCellDataV2Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "AccountCellDataV2Reader".to_string(),
        ))
    }

    fn try_into_v3(&self) -> VerificationResult<AccountCellDataV3Reader<'r>> {
        AccountCellDataV3Reader::from_slice(self.as_slice())
    }

    fn try_into_latest(&self) -> VerificationResult<AccountCellDataReader<'r>> {
        Err(VerificationError::OffsetsNotMatch("AccountCellDataReader".to_string()))
    }

    gen_impl_field_fns!({
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        registered_at -> Uint64Reader<'r>,
        last_transfer_account_at -> Uint64Reader<'r>,
        last_edit_manager_at -> Uint64Reader<'r>,
        last_edit_records_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>
    });
}

impl<'r> AccountCellDataReaderMixer<'r> for AccountCellDataReader<'r> {
    fn version(&self) -> u32 {
        4
    }

    fn try_into_v2(&self) -> VerificationResult<AccountCellDataV2Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "AccountCellDataV2Reader".to_string(),
        ))
    }

    fn try_into_v3(&self) -> VerificationResult<AccountCellDataV3Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "AccountCellDataV3Reader".to_string(),
        ))
    }

    fn try_into_latest(&self) -> VerificationResult<AccountCellDataReader<'r>> {
        AccountCellDataReader::from_slice(self.as_slice())
    }

    gen_impl_field_fns!({
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        registered_at -> Uint64Reader<'r>,
        last_transfer_account_at -> Uint64Reader<'r>,
        last_edit_manager_at -> Uint64Reader<'r>,
        last_edit_records_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>
    });
}

pub trait AccountSaleCellDataMixer {
    gen_trait_common_fns!({
        version -> u32,
        as_reader -> Box<dyn AccountSaleCellDataReaderMixer + '_>
    });
}

impl AccountSaleCellDataMixer for AccountSaleCellDataV1 {
    fn version(&self) -> u32 {
        1
    }

    fn as_reader(&self) -> Box<dyn AccountSaleCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

impl AccountSaleCellDataMixer for AccountSaleCellData {
    fn version(&self) -> u32 {
        2
    }

    fn as_reader(&self) -> Box<dyn AccountSaleCellDataReaderMixer + '_> {
        Box::new(self.as_reader())
    }
}

pub trait AccountSaleCellDataReaderMixer<'r> {
    gen_trait_common_fns!({
        version -> u32,
        try_into_v1 -> VerificationResult<AccountSaleCellDataV1Reader<'r>>,
        try_into_latest -> VerificationResult<AccountSaleCellDataReader<'r>>
    });

    gen_trait_field_fns!({
        account_id -> AccountIdReader<'r>,
        account -> BytesReader<'r>,
        price -> Uint64Reader<'r>,
        description -> BytesReader<'r>,
        started_at -> Uint64Reader<'r>
    });
}

impl<'r> AccountSaleCellDataReaderMixer<'r> for AccountSaleCellDataV1Reader<'r> {
    fn version(&self) -> u32 {
        1
    }

    fn try_into_v1(&self) -> VerificationResult<AccountSaleCellDataV1Reader<'r>> {
        AccountSaleCellDataV1Reader::from_slice(self.as_slice())
    }

    fn try_into_latest(&self) -> VerificationResult<AccountSaleCellDataReader<'r>> {
        Err(VerificationError::OffsetsNotMatch("AccountCellDataReader".to_string()))
    }

    gen_impl_field_fns!({
        account_id -> AccountIdReader<'r>,
        account -> BytesReader<'r>,
        price -> Uint64Reader<'r>,
        description -> BytesReader<'r>,
        started_at -> Uint64Reader<'r>
    });
}

impl<'r> AccountSaleCellDataReaderMixer<'r> for AccountSaleCellDataReader<'r> {
    fn version(&self) -> u32 {
        2
    }

    fn try_into_v1(&self) -> VerificationResult<AccountSaleCellDataV1Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch(
            "AccountCellDataV2Reader".to_string(),
        ))
    }

    fn try_into_latest(&self) -> VerificationResult<AccountSaleCellDataReader<'r>> {
        AccountSaleCellDataReader::from_slice(self.as_slice())
    }

    gen_impl_field_fns!({
        account_id -> AccountIdReader<'r>,
        account -> BytesReader<'r>,
        price -> Uint64Reader<'r>,
        description -> BytesReader<'r>,
        started_at -> Uint64Reader<'r>
    });
}

pub trait SubAccountMixer {
    gen_trait_common_fns!({
        version -> u32,
        as_reader -> Box<dyn SubAccountReaderMixer + '_>,
        as_slice -> &[u8],
        try_into_v1 -> VerificationResult<SubAccountV1>,
        try_into_latest -> VerificationResult<SubAccount>,
        clone -> Box<dyn SubAccountMixer>,
        as_prettier -> String
    });
}

impl SubAccountMixer for SubAccountV1 {
    fn version(&self) -> u32 {
        1
    }

    fn as_reader(&self) -> Box<dyn SubAccountReaderMixer + '_> {
        Box::new(self.as_reader())
    }

    fn as_slice(&self) -> &[u8] {
        Entity::as_slice(self)
    }

    fn try_into_v1(&self) -> VerificationResult<SubAccountV1> {
        SubAccountV1::from_slice(Entity::as_slice(self))
    }

    fn try_into_latest(&self) -> VerificationResult<SubAccount> {
        Err(VerificationError::OffsetsNotMatch("SubAccount".to_string()))
    }

    fn clone(&self) -> Box<dyn SubAccountMixer> {
        Box::new(Clone::clone(self))
    }

    fn as_prettier(&self) -> String {
        Prettier::as_prettier(self)
    }
}

impl SubAccountMixer for SubAccount {
    fn version(&self) -> u32 {
        2
    }

    fn as_reader(&self) -> Box<dyn SubAccountReaderMixer + '_> {
        Box::new(self.as_reader())
    }

    fn as_slice(&self) -> &[u8] {
        Entity::as_slice(self)
    }

    fn try_into_v1(&self) -> VerificationResult<SubAccountV1> {
        Err(VerificationError::OffsetsNotMatch("SubAccount".to_string()))
    }

    fn try_into_latest(&self) -> VerificationResult<SubAccount> {
        SubAccount::from_slice(Entity::as_slice(self))
    }

    fn clone(&self) -> Box<dyn SubAccountMixer> {
        Box::new(Clone::clone(self))
    }

    fn as_prettier(&self) -> String {
        Prettier::as_prettier(self)
    }
}

pub trait SubAccountReaderMixer<'r> {
    gen_trait_common_fns!({
        version -> u32,
        try_into_v1 -> VerificationResult<SubAccountV1Reader<'r>>,
        try_into_latest -> VerificationResult<SubAccountReader<'r>>,
        as_slice -> &[u8],
        as_prettier -> String
    });

    gen_trait_field_fns!({
        lock -> ScriptReader<'r>,
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        suffix -> BytesReader<'r>,
        registered_at -> Uint64Reader<'r>,
        expired_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>,
        nonce -> Uint64Reader<'r>,
        enable_sub_account -> Uint8Reader<'r>,
        renew_sub_account_price -> Uint64Reader<'r>
    });
}

impl<'r> SubAccountReaderMixer<'r> for SubAccountV1Reader<'r> {
    fn version(&self) -> u32 {
        1
    }

    fn try_into_v1(&self) -> VerificationResult<SubAccountV1Reader<'r>> {
        SubAccountV1Reader::from_slice(Reader::as_slice(self))
    }

    fn try_into_latest(&self) -> VerificationResult<SubAccountReader<'r>> {
        Err(VerificationError::OffsetsNotMatch("SubAccountReader".to_string()))
    }

    fn as_slice(&self) -> &[u8] {
        Reader::as_slice(self)
    }

    fn as_prettier(&self) -> String {
        Prettier::as_prettier(self)
    }

    gen_impl_field_fns!({
        lock -> ScriptReader<'r>,
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        suffix -> BytesReader<'r>,
        registered_at -> Uint64Reader<'r>,
        expired_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>,
        nonce -> Uint64Reader<'r>,
        enable_sub_account -> Uint8Reader<'r>,
        renew_sub_account_price -> Uint64Reader<'r>
    });
}

impl<'r> SubAccountReaderMixer<'r> for SubAccountReader<'r> {
    fn version(&self) -> u32 {
        1
    }

    fn try_into_v1(&self) -> VerificationResult<SubAccountV1Reader<'r>> {
        Err(VerificationError::OffsetsNotMatch("SubAccountV1Reader".to_string()))
    }

    fn try_into_latest(&self) -> VerificationResult<SubAccountReader<'r>> {
        SubAccountReader::from_slice(Reader::as_slice(self))
    }

    fn as_slice(&self) -> &[u8] {
        Reader::as_slice(self)
    }

    fn as_prettier(&self) -> String {
        Prettier::as_prettier(self)
    }

    gen_impl_field_fns!({
        lock -> ScriptReader<'r>,
        id -> AccountIdReader<'r>,
        account -> AccountCharsReader<'r>,
        suffix -> BytesReader<'r>,
        registered_at -> Uint64Reader<'r>,
        expired_at -> Uint64Reader<'r>,
        status -> Uint8Reader<'r>,
        records -> RecordsReader<'r>,
        nonce -> Uint64Reader<'r>,
        enable_sub_account -> Uint8Reader<'r>,
        renew_sub_account_price -> Uint64Reader<'r>
    });
}
