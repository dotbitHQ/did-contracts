use super::{assert, constants::ScriptHashType, debug, error::Error, util, warn};
use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::convert::TryFrom;
use core::lazy::OnceCell;
use das_types::{
    constants::{
        CharSetType, DataType, CHAR_SET_LENGTH, WITNESS_HEADER_BYTES, WITNESS_LENGTH_BYTES, WITNESS_TYPE_BYTES,
    },
    packed::*,
    prelude::Entity,
    util as das_types_util,
};

macro_rules! get_or_try_init {
    ( $self:expr, $property:ident, $entity_type:ty, $data_type:expr ) => {{
        $self
            .$property
            .get_or_try_init(|| {
                let (i, raw) = Configs::parse_witness(&$self.config_witnesses, $data_type)?;
                let entity = <$entity_type>::from_slice(&raw).map_err(|e| {
                    warn!("witnesses[{}] Decoding {:?} failed: {}", i, $data_type, e);
                    Error::ConfigCellWitnessDecodingError
                })?;

                Ok(entity)
            })
            .map(|entity| entity.as_reader())
    }};
}

pub struct ScriptLiteral {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub args: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CharSet {
    pub name: CharSetType,
    pub global: bool,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct LockScriptTypeIdTable {
    pub always_success: Script,
    pub das_lock: Script,
    pub secp256k1_blake160_signhash_all: Script,
    pub secp256k1_blake160_multisig_all: Script,
}

#[derive(Debug)]
pub struct Configs {
    config_witnesses: BTreeMap<u32, (usize, [u8; 32])>,
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
    pub record_key_namespace: OnceCell<Vec<u8>>,
    pub preserved_account: OnceCell<Vec<u8>>,
    pub unavailable_account: OnceCell<Vec<u8>>,
    pub sub_account_beta_list: OnceCell<Vec<u8>>,
}

impl Configs {
    pub fn new(config_witnesses: BTreeMap<u32, (usize, [u8; 32])>) -> Self {
        Configs {
            config_witnesses,
            account: OnceCell::new(),
            apply: OnceCell::new(),
            // Chinese charsets is still not enabled.
            char_set: vec![OnceCell::new(); CHAR_SET_LENGTH - 2],
            income: OnceCell::new(),
            main: OnceCell::new(),
            price: OnceCell::new(),
            proposal: OnceCell::new(),
            profit_rate: OnceCell::new(),
            release: OnceCell::new(),
            secondary_market: OnceCell::new(),
            reverse_resolution: OnceCell::new(),
            sub_account: OnceCell::new(),
            record_key_namespace: OnceCell::new(),
            preserved_account: OnceCell::new(),
            unavailable_account: OnceCell::new(),
            sub_account_beta_list: OnceCell::new(),
        }
    }

    fn parse_witness(
        config_witnesses: &BTreeMap<u32, (usize, [u8; 32])>,
        data_type: DataType,
    ) -> Result<(usize, Vec<u8>), Error> {
        let &(i, expected_hash) = config_witnesses.get(&(data_type as u32)).ok_or_else(|| {
            warn!("Can not find {:?} in witnesses.", data_type);
            Error::ConfigIsPartialMissing
        })?;

        debug!("Parsing witnesses[{}] as {:?} ...", i, data_type);

        let raw = util::load_das_witnesses(i)?;
        let entity = raw
            .get((WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES)..)
            .ok_or(Error::ConfigCellWitnessDecodingError)?;
        let hash = util::blake2b_256(entity);

        assert!(
            hash == expected_hash,
            Error::ConfigCellWitnessIsCorrupted,
            "witnesses[{}] The witness is corrupted!(expected_hash: 0x{} current_hash: 0x{})",
            i,
            util::hex_string(&expected_hash),
            util::hex_string(&hash)
        );

        Ok((i, entity.to_vec()))
    }

    pub fn account(&self) -> Result<ConfigCellAccountReader, Error> {
        get_or_try_init!(self, account, ConfigCellAccount, DataType::ConfigCellAccount)
    }

    pub fn apply(&self) -> Result<ConfigCellApplyReader, Error> {
        get_or_try_init!(self, apply, ConfigCellApply, DataType::ConfigCellApply)
    }

    pub fn income(&self) -> Result<ConfigCellIncomeReader, Error> {
        get_or_try_init!(self, income, ConfigCellIncome, DataType::ConfigCellIncome)
    }

    pub fn main(&self) -> Result<ConfigCellMainReader, Error> {
        get_or_try_init!(self, main, ConfigCellMain, DataType::ConfigCellMain)
    }

    pub fn price(&self) -> Result<ConfigCellPriceReader, Error> {
        get_or_try_init!(self, price, ConfigCellPrice, DataType::ConfigCellPrice)
    }

    pub fn proposal(&self) -> Result<ConfigCellProposalReader, Error> {
        get_or_try_init!(self, proposal, ConfigCellProposal, DataType::ConfigCellProposal)
    }

    pub fn profit_rate(&self) -> Result<ConfigCellProfitRateReader, Error> {
        get_or_try_init!(self, profit_rate, ConfigCellProfitRate, DataType::ConfigCellProfitRate)
    }

    pub fn release(&self) -> Result<ConfigCellReleaseReader, Error> {
        get_or_try_init!(self, release, ConfigCellRelease, DataType::ConfigCellRelease)
    }

    pub fn secondary_market(&self) -> Result<ConfigCellSecondaryMarketReader, Error> {
        get_or_try_init!(
            self,
            secondary_market,
            ConfigCellSecondaryMarket,
            DataType::ConfigCellSecondaryMarket
        )
    }

    pub fn reverse_resolution(&self) -> Result<ConfigCellReverseResolutionReader, Error> {
        get_or_try_init!(
            self,
            reverse_resolution,
            ConfigCellReverseResolution,
            DataType::ConfigCellReverseResolution
        )
    }

    pub fn sub_account(&self) -> Result<ConfigCellSubAccountReader, Error> {
        get_or_try_init!(self, sub_account, ConfigCellSubAccount, DataType::ConfigCellSubAccount)
    }

    pub fn record_key_namespace(&self) -> Result<&Vec<u8>, Error> {
        self.record_key_namespace.get_or_try_init(|| {
            let data_type = DataType::ConfigCellRecordKeyNamespace;
            let (i, raw) = Self::parse_witness(&self.config_witnesses, data_type)?;
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("witnesses[{}] The data of {:?} is empty.", i, data_type);
                    return Err(Error::ConfigIsPartialMissing);
                }
            };

            Ok(data)
        })
    }

    pub fn preserved_account(&self, data_type: DataType) -> Result<&Vec<u8>, Error> {
        self.preserved_account.get_or_try_init(|| {
            let (i, raw) = Self::parse_witness(&self.config_witnesses, data_type)?;
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("witnesses[{}] The data of {:?} is empty.", i, data_type);

                    return Err(Error::ConfigIsPartialMissing);
                }
            };

            Ok(data)
        })
    }

    pub fn unavailable_account(&self) -> Result<&Vec<u8>, Error> {
        self.unavailable_account.get_or_try_init(|| {
            let data_type = DataType::ConfigCellUnAvailableAccount;
            let (i, raw) = Self::parse_witness(&self.config_witnesses, data_type)?;
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("witnesses[{}] The data of {:?} is empty.", i, data_type);
                    return Err(Error::ConfigIsPartialMissing);
                }
            };

            Ok(data)
        })
    }

    pub fn sub_account_beta_list(&self) -> Result<&Vec<u8>, Error> {
        self.unavailable_account.get_or_try_init(|| {
            let data_type = DataType::ConfigCellSubAccountBetaList;
            let (i, raw) = Self::parse_witness(&self.config_witnesses, data_type)?;
            let data = match raw.get(WITNESS_LENGTH_BYTES..) {
                Some(data) => data.to_vec(),
                None => {
                    warn!("witnesses[{}] The data of {:?} is empty.", i, data_type);
                    return Err(Error::ConfigIsPartialMissing);
                }
            };

            Ok(data)
        })
    }

    pub fn char_set(&self) -> Vec<Result<&CharSet, Error>> {
        let mut ret = Vec::new();
        for (i, char_set) in self.char_set.iter().enumerate() {
            let item = char_set.get_or_try_init(|| {
                let char_set_type = match CharSetType::try_from(i as u32) {
                    Ok(char_set_type) => char_set_type,
                    Err(_) => {
                        warn!("Invalid CharSetType[{}]", i);
                        return Err(Error::ConfigCellWitnessDecodingError);
                    }
                };
                let data_type = das_types_util::char_set_to_data_type(char_set_type);
                let (i, raw) = Self::parse_witness(&self.config_witnesses, data_type)?;
                let length = match raw.get(..WITNESS_LENGTH_BYTES) {
                    Some(length_bytes) => {
                        let mut tmp = [0u8; 4];
                        tmp.copy_from_slice(length_bytes);
                        u32::from_le_bytes(tmp) as usize
                    }
                    None => {
                        warn!("witnesses[{}] The data of {:?} is empty.", i, data_type);
                        return Err(Error::ConfigIsPartialMissing);
                    }
                };

                assert!(
                    raw.len() == length,
                    Error::ConfigCellWitnessDecodingError,
                    "witnesses[{}] The {:?} should have length of {} bytes, but {} bytes found.",
                    i,
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
            });

            ret.push(item);
        }

        ret
    }
}
