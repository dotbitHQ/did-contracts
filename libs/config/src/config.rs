#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::cell::{OnceCell, Ref, RefCell};
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_types::constants::*;

use super::error::ConfigError;
use crate::configs::char_set::ConfigCharSet;
use crate::configs::entity_config::{
    ConfigAccount, ConfigApply, ConfigDPoint, ConfigIncome, ConfigPrice, ConfigProfitRate, ConfigProposal,
    ConfigRelease, ConfigReverseResolution, ConfigSecondaryMarket, ConfigSubAccount,
};
use crate::configs::main::ConfigMain;
use crate::configs::preserved_account::ConfigPreservedAccount;
use crate::configs::raw_config::{ConfigRecordKeyNamespace, ConfigUnavailableAccount};
use crate::configs::smt_node_white_list::ConfigSMTNodeWhiteList;
use crate::traits::SerializableConfig;

#[derive(Clone, Debug)]
pub struct CharSet {
    pub name: CharSetType,
    pub global: bool,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Config {
    pub config_cells: BTreeMap<DataType, usize>,

    pub main: RefCell<ConfigMain>,
    // Entity configs
    pub account: RefCell<ConfigAccount>,
    pub apply: RefCell<ConfigApply>,
    pub income: RefCell<ConfigIncome>,
    pub price: RefCell<ConfigPrice>,
    pub proposal: RefCell<ConfigProposal>,
    pub profit_rate: RefCell<ConfigProfitRate>,
    pub release: RefCell<ConfigRelease>,
    pub secondary_market: RefCell<ConfigSecondaryMarket>,
    pub reverse_resolution: RefCell<ConfigReverseResolution>,
    pub sub_account: RefCell<ConfigSubAccount>,
    pub dpoint: RefCell<ConfigDPoint>,
    // Raw configs
    pub char_set: Vec<RefCell<ConfigCharSet>>,
    pub record_key_namespace: RefCell<ConfigRecordKeyNamespace>,
    pub preserved_account: RefCell<ConfigPreservedAccount>,
    pub unavailable_account: RefCell<ConfigUnavailableAccount>,
    pub smt_node_white_list: RefCell<ConfigSMTNodeWhiteList>,
}

impl Config {
    fn new(config_cells: BTreeMap<DataType, usize>) -> Self {
        Self {
            config_cells,

            main: RefCell::new(ConfigMain::default()),
            // Entity configs
            account: RefCell::new(ConfigAccount::default()),
            apply: RefCell::new(ConfigApply::default()),
            income: RefCell::new(ConfigIncome::default()),
            price: RefCell::new(ConfigPrice::default()),
            proposal: RefCell::new(ConfigProposal::default()),
            profit_rate: RefCell::new(ConfigProfitRate::default()),
            release: RefCell::new(ConfigRelease::default()),
            secondary_market: RefCell::new(ConfigSecondaryMarket::default()),
            reverse_resolution: RefCell::new(ConfigReverseResolution::default()),
            sub_account: RefCell::new(ConfigSubAccount::default()),
            dpoint: RefCell::new(ConfigDPoint::default()),
            // Raw configs
            char_set: vec![RefCell::new(ConfigCharSet::default()); CHAR_SET_LENGTH],
            record_key_namespace: RefCell::new(ConfigRecordKeyNamespace::default()),
            preserved_account: RefCell::new(ConfigPreservedAccount::default()),
            unavailable_account: RefCell::new(ConfigUnavailableAccount::default()),
            smt_node_white_list: RefCell::new(ConfigSMTNodeWhiteList::default()),
        }
    }
}

impl Config {
    pub fn get_instance() -> &'static mut Self {
        static mut CONFIG: OnceCell<Config> = OnceCell::new();
        unsafe {
            CONFIG.get_or_init(|| {
                let config_cells = match Self::find_config_cells_by_type_id() {
                    Ok(cells) => cells,
                    Err(err) => {
                        warn!("Failed to find config cells by type id: {:?}", err);
                        panic!();
                    }
                };

                let res = Self::new(config_cells);
                res
            });
            CONFIG.get_mut().unwrap()
        }
    }

    pub fn main(&self) -> Result<Ref<ConfigMain>, ConfigError> {
        if !self.main.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellMain)?;
            self.main.replace(config);
        }

        Ok(self.main.borrow())
    }

    pub fn account(&self) -> Result<Ref<ConfigAccount>, ConfigError> {
        if !self.account.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellAccount)?;
            self.account.replace(config);
        }

        Ok(self.account.borrow())
    }

    pub fn apply(&self) -> Result<Ref<ConfigApply>, ConfigError> {
        if !self.apply.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellApply)?;
            self.apply.replace(config);
        }

        Ok(self.apply.borrow())
    }

    pub fn dpoint(&self) -> Result<Ref<ConfigDPoint>, ConfigError> {
        if !self.dpoint.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellDPoint)?;
            self.dpoint.replace(config);
        }

        Ok(self.dpoint.borrow())
    }

    pub fn income(&self) -> Result<Ref<ConfigIncome>, ConfigError> {
        if !self.income.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellIncome)?;
            self.income.replace(config);
        }

        Ok(self.income.borrow())
    }

    pub fn price(&self) -> Result<Ref<ConfigPrice>, ConfigError> {
        if !self.price.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellPrice)?;
            self.price.replace(config);
        }

        Ok(self.price.borrow())
    }

    pub fn profit_rate(&self) -> Result<Ref<ConfigProfitRate>, ConfigError> {
        if !self.profit_rate.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellProfitRate)?;
            self.profit_rate.replace(config);
        }

        Ok(self.profit_rate.borrow())
    }

    pub fn proposal(&self) -> Result<Ref<ConfigProposal>, ConfigError> {
        if !self.proposal.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellProposal)?;
            self.proposal.replace(config);
        }

        Ok(self.proposal.borrow())
    }

    pub fn release(&self) -> Result<Ref<ConfigRelease>, ConfigError> {
        if !self.release.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellRelease)?;
            self.release.replace(config);
        }

        Ok(self.release.borrow())
    }

    pub fn reverse_resolution(&self) -> Result<Ref<ConfigReverseResolution>, ConfigError> {
        if !self.reverse_resolution.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellReverseResolution)?;
            self.reverse_resolution.replace(config);
        }

        Ok(self.reverse_resolution.borrow())
    }

    pub fn secondary_market(&self) -> Result<Ref<ConfigSecondaryMarket>, ConfigError> {
        if !self.secondary_market.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellSecondaryMarket)?;
            self.secondary_market.replace(config);
        }

        Ok(self.secondary_market.borrow())
    }

    pub fn sub_account(&self) -> Result<Ref<ConfigSubAccount>, ConfigError> {
        if !self.sub_account.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellSubAccount)?;
            self.sub_account.replace(config);
        }

        Ok(self.sub_account.borrow())
    }

    pub fn preserved_account(&self, data_type: DataType) -> Result<Ref<ConfigPreservedAccount>, ConfigError> {
        if !self.preserved_account.borrow().inited {
            let index = match self.config_cells.get(&data_type) {
                Some(index) => *index,
                None => {
                    return Err(ConfigError::ConfigCellNotFound { data_type });
                }
            };

            let data = high_level::load_cell_data(index, Source::CellDep)
                .map_err(|_err| ConfigError::LoadCellDataError { index })?;
            let version = data[0];
            let value = &data[1..];

            let config = ConfigPreservedAccount::new(data_type, version, value)?;

            self.preserved_account.replace(config);
        }

        Ok(self.preserved_account.borrow())
    }

    pub fn unavailable_account(&self) -> Result<Ref<ConfigUnavailableAccount>, ConfigError> {
        if !self.unavailable_account.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellUnAvailableAccount)?;
            self.unavailable_account.replace(config);
        }

        Ok(self.unavailable_account.borrow())
    }

    pub fn record_key_namespace(&self) -> Result<Ref<ConfigRecordKeyNamespace>, ConfigError> {
        if !self.record_key_namespace.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellRecordKeyNamespace)?;
            self.record_key_namespace.replace(config);
        }

        Ok(self.record_key_namespace.borrow())
    }

    pub fn char_set(&self, char_set_type: CharSetType) -> Result<Ref<ConfigCharSet>, ConfigError> {
        let index = char_set_type as u32 as usize;
        match self.char_set.get(index) {
            None => return Err(ConfigError::UndefinedCharSetType { index: index as u32 }),
            Some(chat_set) => {
                if !chat_set.borrow().inited {
                    let data_type = ConfigCharSet::char_set_type_to_data_type(char_set_type);
                    let cell_index = match self.config_cells.get(&data_type) {
                        Some(index) => *index,
                        None => {
                            return Err(ConfigError::ConfigCellNotFound { data_type });
                        }
                    };

                    let raw = high_level::load_cell_data(cell_index, Source::CellDep)
                        .map_err(|_err| ConfigError::LoadCellDataError { index: cell_index })?;

                    let version = raw[0];
                    let is_global = raw[1] == 1u8;
                    let value = &raw[2..];
                    let config = ConfigCharSet::new(data_type, version, char_set_type, is_global, value)?;

                    self.char_set[index].replace(config);
                }
            }
        }

        Ok(self.char_set.get(index).unwrap().borrow())
    }

    pub fn smt_node_white_list(&self) -> Result<Ref<ConfigSMTNodeWhiteList>, ConfigError> {
        if !self.smt_node_white_list.borrow().inited {
            let config = self.load_config_from_cell_data(DataType::ConfigCellSMTNodeWhitelist)?;
            self.smt_node_white_list.replace(config);
        }

        Ok(self.smt_node_white_list.borrow())
    }

    fn find_config_cells_by_type_id() -> Result<BTreeMap<DataType, usize>, ConfigError> {
        let config_cell_type = config_cell_type();

        let mut data_type_map = BTreeMap::new();

        high_level::QueryIter::new(high_level::load_cell_type, Source::CellDep)
            .enumerate()
            .filter(|(_i, script_opt)| match script_opt {
                None => false,
                Some(script) => {
                    config_cell_type.code_hash().as_reader().raw_data() == script.code_hash().as_reader().raw_data()
                        && config_cell_type.hash_type().as_slice() == script.hash_type().as_slice()
                }
            })
            .try_for_each(|(i, script_opt)| {
                let script = script_opt.unwrap();
                let data_type_in_int = u32::from_le_bytes(script.args().as_reader().raw_data().try_into().unwrap());

                let data_type =
                    DataType::try_from(data_type_in_int).map_err(|_err| ConfigError::UndefinedDataType {
                        index: i,
                        data_type: data_type_in_int,
                    })?;

                data_type_map.insert(data_type, i);

                Ok(())
            })?;

        Ok(data_type_map)
    }

    fn load_config_from_cell_data<T: SerializableConfig>(&self, data_type: DataType) -> Result<T, ConfigError> {
        let index = match self.config_cells.get(&data_type) {
            Some(index) => *index,
            None => {
                return Err(ConfigError::ConfigCellNotFound { data_type });
            }
        };
        let data = high_level::load_cell_data(index, Source::CellDep)
            .map_err(|_err| ConfigError::LoadCellDataError { index })?;

        T::from_slice(&data)
    }
}
