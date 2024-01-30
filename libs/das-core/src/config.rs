use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::OnceCell;
use core::convert::TryFrom;

use das_types::constants::*;
use das_types::packed::*;
use das_types::util as das_types_util;
use molecule::prelude::Entity;
use witness_parser::traits::WitnessQueryable;
use witness_parser::WitnessesParserV1;

use super::error::*;
use super::{assert, code_to_error, warn};
use crate::types::CharSet;

macro_rules! get_or_try_init {
    ( $self:expr, $property:ident, $entity_type:ty, $data_type:expr ) => {{
        $self
            .$property
            .get_or_try_init(|| {
                let entity = $self.parse_witness($data_type)?;
                Ok(entity)
            })
            .map(|entity| entity.as_reader())
    }};
}

#[derive(Debug)]
pub struct Config {
    pub account: OnceCell<ConfigCellAccount>,
    pub apply: OnceCell<ConfigCellApply>,
    pub char_set: Vec<OnceCell<CharSet>>,
    pub income: OnceCell<ConfigCellIncome>,
    pub main: OnceCell<ConfigCellMain>,
    pub price: OnceCell<ConfigCellPrice>,
    pub proposal: OnceCell<ConfigCellProposal>,
    pub profit_rate: OnceCell<ConfigCellProfitRate>,
    pub release: OnceCell<ConfigCellRelease>,
    pub secondary_market: OnceCell<ConfigCellSecondaryMarket>,
    pub reverse_resolution: OnceCell<ConfigCellReverseResolution>,
    pub sub_account: OnceCell<ConfigCellSubAccount>,
    pub dpoint: OnceCell<ConfigCellDPoint>,
    pub record_key_namespace: OnceCell<Vec<u8>>,
    pub preserved_account: OnceCell<Vec<u8>>,
    pub unavailable_account: OnceCell<Vec<u8>>,
    pub smt_node_white_list: OnceCell<Vec<[u8; 32]>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            account: OnceCell::new(),
            apply: OnceCell::new(),
            char_set: vec![OnceCell::new(); CHAR_SET_LENGTH],
            income: OnceCell::new(),
            main: OnceCell::new(),
            price: OnceCell::new(),
            proposal: OnceCell::new(),
            profit_rate: OnceCell::new(),
            release: OnceCell::new(),
            secondary_market: OnceCell::new(),
            reverse_resolution: OnceCell::new(),
            sub_account: OnceCell::new(),
            dpoint: OnceCell::new(),
            record_key_namespace: OnceCell::new(),
            preserved_account: OnceCell::new(),
            unavailable_account: OnceCell::new(),
            smt_node_white_list: OnceCell::new(),
        }
    }
}

impl Config {
    pub fn get_instance() -> &'static mut Self {
        static mut CONFIG: OnceCell<Config> = OnceCell::new();
        unsafe {
            CONFIG.get_or_init(|| {
                let res = Self::default();
                // TODO Try a better way to implement singleton to handle the init errors.
                // res.init().unwrap();
                res
            });
            CONFIG.get_mut().unwrap()
        }
    }

    fn parse_witness<T: Entity>(&self, config_id: DataType) -> Result<T, Box<dyn ScriptError>> {
        let parser = WitnessesParserV1::get_instance();
        if !parser.is_inited() {
            return Err(code_to_error!(ErrorCode::WitnessNotInited));
        }

        let entity: T = parser
            .get_entity_by_data_type(config_id)
            .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

        Ok(entity)
    }

    fn parse_raw_witness(&self, config_id: DataType) -> Result<Vec<u8>, Box<dyn ScriptError>> {
        let parser = WitnessesParserV1::get_instance();
        if !parser.is_inited() {
            return Err(code_to_error!(ErrorCode::WitnessNotInited));
        }

        let mut raw = parser
            .get_raw_by_data_type(config_id)
            .map_err(|_err| code_to_error!(ErrorCode::ConfigIsPartialMissing))?;
        raw.drain(..(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES));

        Ok(raw)
    }

    pub fn account(&self) -> Result<ConfigCellAccountReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, account, ConfigCellAccount, DataType::ConfigCellAccount)
    }

    pub fn apply(&self) -> Result<ConfigCellApplyReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, apply, ConfigCellApply, DataType::ConfigCellApply)
    }

    pub fn income(&self) -> Result<ConfigCellIncomeReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, income, ConfigCellIncome, DataType::ConfigCellIncome)
    }

    pub fn main(&self) -> Result<ConfigCellMainReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, main, ConfigCellMain, DataType::ConfigCellMain)
    }

    pub fn price(&self) -> Result<ConfigCellPriceReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, price, ConfigCellPrice, DataType::ConfigCellPrice)
    }

    pub fn proposal(&self) -> Result<ConfigCellProposalReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, proposal, ConfigCellProposal, DataType::ConfigCellProposal)
    }

    pub fn profit_rate(&self) -> Result<ConfigCellProfitRateReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, profit_rate, ConfigCellProfitRate, DataType::ConfigCellProfitRate)
    }

    pub fn release(&self) -> Result<ConfigCellReleaseReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, release, ConfigCellRelease, DataType::ConfigCellRelease)
    }

    pub fn secondary_market(&self) -> Result<ConfigCellSecondaryMarketReader, Box<dyn ScriptError>> {
        get_or_try_init!(
            self,
            secondary_market,
            ConfigCellSecondaryMarket,
            DataType::ConfigCellSecondaryMarket
        )
    }

    pub fn reverse_resolution(&self) -> Result<ConfigCellReverseResolutionReader, Box<dyn ScriptError>> {
        get_or_try_init!(
            self,
            reverse_resolution,
            ConfigCellReverseResolution,
            DataType::ConfigCellReverseResolution
        )
    }

    pub fn sub_account(&self) -> Result<ConfigCellSubAccountReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, sub_account, ConfigCellSubAccount, DataType::ConfigCellSubAccount)
    }

    pub fn dpoint(&self) -> Result<ConfigCellDPointReader, Box<dyn ScriptError>> {
        get_or_try_init!(self, dpoint, ConfigCellDPoint, DataType::ConfigCellDPoint)
    }

    pub fn record_key_namespace(&self) -> Result<&Vec<u8>, Box<dyn ScriptError>> {
        self.record_key_namespace.get_or_try_init(|| {
            let data_type = DataType::ConfigCellRecordKeyNamespace;
            let raw = self.parse_raw_witness(data_type)?;
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("The data of {:?} is empty.", data_type);
                    return Err(code_to_error!(ErrorCode::ConfigIsPartialMissing).into());
                }
            };

            Ok(data)
        })
    }

    pub fn preserved_account(&self, data_type: DataType) -> Result<&Vec<u8>, Box<dyn ScriptError>> {
        self.preserved_account.get_or_try_init(|| {
            let raw = match data_type {
                DataType::ConfigCellPreservedAccount00 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount01 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount02 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount03 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount04 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount05 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount06 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount07 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount08 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount09 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount10 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount11 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount12 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount13 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount14 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount15 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount16 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount17 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount18 => self.parse_raw_witness(data_type)?,
                DataType::ConfigCellPreservedAccount19 => self.parse_raw_witness(data_type)?,
                _ => {
                    warn!("Invalid preserved account data type: {:?}", data_type);
                    return Err(code_to_error!(ErrorCode::WitnessDataTypeDecodingError).into());
                }
            };
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("The data of {:?} is empty.", data_type);

                    return Err(code_to_error!(ErrorCode::ConfigIsPartialMissing).into());
                }
            };

            Ok(data)
        })
    }

    pub fn unavailable_account(&self) -> Result<&Vec<u8>, Box<dyn ScriptError>> {
        self.unavailable_account.get_or_try_init(|| {
            let data_type = DataType::ConfigCellUnAvailableAccount;
            let raw = self.parse_raw_witness(data_type)?;
            debug!("raw: {:?}", hex::encode(&raw));
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("The data of {:?} is empty.", data_type);
                    return Err(code_to_error!(ErrorCode::ConfigIsPartialMissing).into());
                }
            };

            Ok(data)
        })
    }

    pub fn char_set(&self, char_set_index: usize) -> Option<Result<&CharSet, Box<dyn ScriptError>>> {
        self.char_set.get(char_set_index).map(|char_set| {
            char_set.get_or_try_init(|| {
                let char_set_type = match CharSetType::try_from(char_set_index as u32) {
                    Ok(char_set_type) => char_set_type,
                    Err(_) => {
                        warn!("Invalid CharSetType[{}]", char_set_index);
                        return Err(code_to_error!(ErrorCode::ConfigCellWitnessDecodingError).into());
                    }
                };
                let data_type = das_types_util::char_set_to_data_type(char_set_type);
                let raw = match data_type {
                    DataType::ConfigCellCharSetDigit => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetEmoji => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetEn => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetJa => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetKo => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetRu => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetTh => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetTr => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetVi => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetZhHans => self.parse_raw_witness(data_type)?,
                    DataType::ConfigCellCharSetZhHant => self.parse_raw_witness(data_type)?,
                    _ => {
                        warn!("Invalid preserved account data type: {:?}", data_type);
                        return Err(code_to_error!(ErrorCode::WitnessDataTypeDecodingError).into());
                    }
                };
                let length = match raw.get(..WITNESS_LENGTH_BYTES) {
                    Some(length_bytes) => {
                        let mut tmp = [0u8; 4];
                        tmp.copy_from_slice(length_bytes);
                        u32::from_le_bytes(tmp) as usize
                    }
                    None => {
                        warn!("The data of {:?} is empty.", data_type);
                        return Err(code_to_error!(ErrorCode::ConfigIsPartialMissing).into());
                    }
                };

                assert!(
                    raw.len() == length,
                    ErrorCode::ConfigCellWitnessDecodingError,
                    "The {:?} should have length of {} bytes, but {} bytes found.",
                    data_type,
                    length,
                    raw.len()
                );

                let char_set = CharSet {
                    name: char_set_type,
                    // skip WITNESS_LENGTH_BYTES bytes length, and the WITNESS_LENGTH_BYTES+1 byte is global flag, then the following bytes is data
                    global: raw.get(WITNESS_LENGTH_BYTES).unwrap() == &1u8,
                    data: raw.get((WITNESS_LENGTH_BYTES + 1)..).unwrap().to_vec(),
                };

                Ok(char_set)
            })
        })
    }

    pub fn smt_node_white_list(&self) -> Result<&Vec<[u8; 32]>, Box<dyn ScriptError>> {
        self.smt_node_white_list.get_or_try_init(|| {
            let data_type = DataType::ConfigCellSMTNodeWhitelist;
            let raw = self.parse_raw_witness(data_type)?;
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => {
                    let mut ret = vec![];

                    let mut from: usize = 0;
                    let mut to: usize = 32;
                    loop {
                        match data.get(from..to) {
                            Some(val) => {
                                das_assert!(
                                    val.len() == 32,
                                    ErrorCode::ConfigCellWitnessDecodingError,
                                    "The data of {:?} should be a multiple of 32 bytes.",
                                    data_type
                                );

                                let mut tmp = [0u8; 32];
                                tmp.copy_from_slice(val);
                                ret.push(tmp);

                                from = to;
                                to += 32;
                            }
                            None => break,
                        }
                    }

                    ret
                }
                None => {
                    warn!("The data of {:?} is empty.", data_type);
                    return Err(code_to_error!(ErrorCode::ConfigIsPartialMissing).into());
                }
            };

            Ok(data)
        })
    }
}
