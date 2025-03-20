#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use das_types::constants::DataType;
use das_types::packed::*;
use das_types::util;
use molecule::prelude::Entity;
use paste::paste;

use crate::error::ConfigError;
use crate::traits::SerializableConfig;

macro_rules! define_entity_config {
    ( $name:ident, $entity_type:ident, $data_type:expr ) => {
        #[derive(Debug, Default, Clone)]
        pub struct $name {
            pub version: u8,
            pub value: $entity_type,
            pub inited: bool,
        }

        impl $name {
            const NAME: &'static str = stringify!($name);
            const DATA_TYPE: DataType = $data_type;

            pub fn new(version: u8, value: &[u8]) -> Result<Self, ConfigError> {
                let entity = das_types::packed::$entity_type::from_compatible_slice(value)
                    .map_err(|_| ConfigError::DecodingError { name: Self::NAME })?;

                Ok(Self {
                    version,
                    value: entity,
                    inited: true,
                })
            }

            pub fn name(&self) -> &'static str {
                Self::NAME
            }

            pub fn data_type(&self) -> DataType {
                Self::DATA_TYPE
            }

            pub fn version(&self) -> u8 {
                self.version
            }

            pub fn value(&self) -> &$entity_type {
                &self.value
            }

            paste! {
                pub fn as_reader(&self) -> [<$entity_type Reader>] {
                    self.value.as_reader()
                }
            }
        }

        impl SerializableConfig for $name {
            fn from_slice(data: &[u8]) -> Result<Self, ConfigError> {
                let version = data[0];
                let value = &data[1..];

                Ok(Self::new(version, value)?)
            }

            fn as_slice(&self) -> Vec<u8> {
                let mut data = vec![self.version];
                data.extend_from_slice(self.value.as_slice());
                data
            }
        }
    };
}

macro_rules! define_field_getter {
    ( $field:ident, $type_:ident ) => {
        pub fn $field(&self) -> $type_ {
            $type_::from(self.value().$field().as_reader())
        }
    };
}

define_entity_config!(ConfigAccount, ConfigCellAccount, DataType::ConfigCellAccount);
define_entity_config!(ConfigApply, ConfigCellApply, DataType::ConfigCellApply);
define_entity_config!(ConfigDPoint, ConfigCellDPoint, DataType::ConfigCellDPoint);
define_entity_config!(ConfigIncome, ConfigCellIncome, DataType::ConfigCellIncome);
define_entity_config!(ConfigPrice, ConfigCellPrice, DataType::ConfigCellPrice);
define_entity_config!(ConfigProfitRate, ConfigCellProfitRate, DataType::ConfigCellProfitRate);
define_entity_config!(ConfigProposal, ConfigCellProposal, DataType::ConfigCellProposal);
define_entity_config!(ConfigRelease, ConfigCellRelease, DataType::ConfigCellRelease);
define_entity_config!(
    ConfigReverseResolution,
    ConfigCellReverseResolution,
    DataType::ConfigCellReverseResolution
);
define_entity_config!(
    ConfigSecondaryMarket,
    ConfigCellSecondaryMarket,
    DataType::ConfigCellSecondaryMarket
);
define_entity_config!(ConfigSubAccount, ConfigCellSubAccount, DataType::SubAccount);

impl ConfigAccount {
    define_field_getter!(basic_capacity, u64);
    define_field_getter!(expiration_grace_period, u32);
    define_field_getter!(expiration_auction_period, u32);
    define_field_getter!(expiration_deliver_period, u32);
    define_field_getter!(max_length, u32);
    define_field_getter!(common_fee, u64);
    define_field_getter!(transfer_account_fee, u64);
    define_field_getter!(edit_manager_fee, u64);
    define_field_getter!(edit_records_fee, u64);
    define_field_getter!(transfer_account_throttle, u32);
    define_field_getter!(edit_manager_throttle, u32);
    define_field_getter!(edit_records_throttle, u32);
    define_field_getter!(expiration_auction_start_premiums, u32);
    define_field_getter!(prepared_fee_capacity, u64);
    define_field_getter!(record_size_limit, u32);
}

impl ConfigApply {
    define_field_getter!(apply_min_waiting_block_number, u32);
    define_field_getter!(apply_max_waiting_block_number, u32);
}

impl ConfigDPoint {
    define_field_getter!(basic_capacity, u64);
    define_field_getter!(prepared_fee_capacity, u64);

    pub fn capacity_recycle_whitelist(&self) -> Vec<[u8; 32]> {
        self.value()
            .as_reader()
            .capacity_recycle_whitelist()
            .iter()
            .map(|lock| util::blake2b_256(lock.as_slice()))
            .collect::<Vec<_>>()
    }

    pub fn transfer_whitelist(&self) -> Vec<[u8; 32]> {
        self.value()
            .as_reader()
            .transfer_whitelist()
            .iter()
            .map(|lock| util::blake2b_256(lock.as_slice()))
            .collect::<Vec<_>>()
    }
}

impl ConfigPrice {
    pub fn prices(&self) -> PriceConfigListReader {
        self.value().as_reader().prices()
    }

    pub fn discount(&self) -> DiscountConfigReader {
        self.value().as_reader().discount()
    }
}

impl ConfigIncome {
    pub fn basic_capacity(&self) -> u64 {
        u64::from(self.value().basic_capacity().as_reader())
    }

    pub fn max_records(&self) -> u32 {
        u32::from(self.value().max_records().as_reader())
    }

    pub fn min_transfer_capacity(&self) -> u64 {
        u64::from(self.value().min_transfer_capacity().as_reader())
    }
}

impl ConfigProfitRate {
    define_field_getter!(inviter, u32);
    define_field_getter!(channel, u32);
    define_field_getter!(proposal_create, u32);
    define_field_getter!(proposal_confirm, u32);
    define_field_getter!(sale_das, u32);
    define_field_getter!(sale_buyer_inviter, u32);
    define_field_getter!(sale_buyer_channel, u32);
    define_field_getter!(income_consolidate, u32);
}

impl ConfigProposal {
    define_field_getter!(proposal_max_account_affect, u32);
    define_field_getter!(proposal_max_pre_account_contain, u32);
    define_field_getter!(proposal_min_confirm_interval, u8);
    define_field_getter!(proposal_min_recycle_interval, u8);
}

impl ConfigRelease {
    define_field_getter!(lucky_number, u32);
}

impl ConfigReverseResolution {
    define_field_getter!(common_fee, u64);
    define_field_getter!(record_basic_capacity, u64);
}

impl ConfigSecondaryMarket {
    define_field_getter!(offer_message_bytes_limit, u32);
    define_field_getter!(offer_cell_basic_capacity, u64);
    define_field_getter!(offer_cell_prepared_fee_capacity, u64);

    define_field_getter!(sale_min_price, u64);
    define_field_getter!(sale_description_bytes_limit, u32);
    define_field_getter!(sale_cell_basic_capacity, u64);
    define_field_getter!(sale_cell_prepared_fee_capacity, u64);

    define_field_getter!(common_fee, u64);
}

impl ConfigSubAccount {
    define_field_getter!(basic_capacity, u64);
    define_field_getter!(prepared_fee_capacity, u64);
    define_field_getter!(common_fee, u64);
    define_field_getter!(new_sub_account_price, u64);
    define_field_getter!(renew_sub_account_price, u64);
}
