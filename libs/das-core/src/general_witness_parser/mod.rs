use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::OnceCell;
use core::ops::Deref;
use core::slice::SlicePattern;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{CellOutput, Script};
use ckb_std::high_level::{
    load_cell, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type, load_cell_type_hash, QueryIter,
};
use ckb_std::syscalls::{load_witness, SysError};
use das_types::constants::{DataType, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use das_types::packed::{ConfigList, Data};
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

use crate::error::{ErrorCode, ScriptError};
use crate::traits::Blake2BHash;
// use crate::util::find_only_cell_by_type_id;

#[derive(Default)]
pub struct GeneralWitnessParser {
    witnesses: Vec<Witness>,
    hashes: BTreeMap<[u8; 32], usize>,
}

pub struct PartialWitness {
    pub buf: Vec<u8>,
    pub actual_size: usize,
}

pub struct CompleteWitness {
    pub buf: Vec<u8>,
    pub parsed: bool,
}

pub struct WithMeta<T> {
    pub item: T,
    pub index: usize,
    pub source: Source,
}

pub enum Witness {
    Loading(WithMeta<PartialWitness>),
    Loaded(WithMeta<CompleteWitness>),
}

pub trait FromWitness {
    type Error;
    fn from_witness(witness: &Witness) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn parsable(witness: &Witness) -> bool;
}

pub struct ParsedWithHash<T> {
    pub result: T,
    pub hash: Option<[u8; 32]>,
}

pub enum Condition<'a> {
    LockIs(&'a Script),
    TypeIs(&'a Script),
    CodeHashIs(&'a [u8; 32]),
    LockHash(&'a [u8; 32]),
    TypeHash(&'a [u8; 32]),
    DataIs(&'a [u8]),
}

// enum CacheState<T> {
//     Cached(T),
//     Initital
// }

// struct CacheEntry {
//     type_script: Option<Option<Script>>,
//     type_hash: Option<Option<[u8;32]>>,
//     lock_script: Option<Script>,
//     lock_hash: Option<[u8; 32]>,
//     data: Option<Vec<u8>>,
// }

#[derive(Eq, PartialEq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum CellField {
    Lock,
    Type,
    Data,
}

type SourceRepr = u64;

#[derive(Default)]
pub struct CellIndexer {
    // pub data: BTreeMap<(usize, u64), Vec<u8>>,
    // pub type_hash: BTreeMap<(usize, u64), Option<[u8;32]>>,
    // pub type_script: BTreeMap<(usize, u64), Option<Script>>,
    // pub lock_hash: BTreeMap<(usize, u64), [u8;32]>,
    // pub lock_script: BTreeMap<(usize, u64), Script>,
    // pub cell: BTreeMap<(usize, u64), CellOutput>,
    pub by_hash: BTreeMap<([u8; 32], SourceRepr, CellField), Vec<usize>>,
}
impl CellIndexer {
    pub fn init(&mut self) -> Result<(), SysError> {
        let load_fn = |index, source| {
            let cell = load_cell(index, source)?;
            Ok(WithMeta {
                item: cell,
                index,
                source,
            })
        };
        let mut index_fn = |item: WithMeta<CellOutput>| {
            self.by_hash
                .entry((
                    item.item.lock().code_hash().as_slice().try_into().unwrap(),
                    item.source as u64,
                    CellField::Lock,
                ))
                .and_modify(|v| v.push(item.index))
                .or_insert(vec![item.index]);

            item.item.type_().to_opt().map(|script| {
                self.by_hash
                    .entry((
                        script.code_hash().as_slice().try_into().unwrap(),
                        item.source as u64,
                        CellField::Type,
                    ))
                    .and_modify(|v| v.push(item.index))
                    .or_insert(vec![item.index])
            });

            let mut data = [0; 32];
            let res = ckb_std::syscalls::load_cell_data(&mut data, 0, item.index, item.source);
            match res {
                Ok(_) | Err(SysError::LengthNotEnough(_)) => {
                    self.by_hash
                        .entry((data, item.source as u64, CellField::Data))
                        .and_modify(|v| v.push(item.index))
                        .or_insert(vec![item.index]);
                }
                _ => (),
            }
        };
        QueryIter::new(load_fn, Source::Input).for_each(&mut index_fn);
        QueryIter::new(load_fn, Source::Output).for_each(&mut index_fn);
        QueryIter::new(load_fn, Source::CellDep).for_each(&mut index_fn);

        Ok(())
    }

    pub fn find_by_hash(&self, hash: &[u8; 32], source: Source, field: CellField) -> Option<&[usize]> {
        self.by_hash.get(&(*hash, source as u64, field)).map(|v| v.as_slice())
    }

    // pub fn load_cell(&mut self, index: usize, source: Source) -> Result<&CellOutput, SysError> {
    //     let cache = self.cell.entry((index, source as u64));
    //     if let Entry::Vacant(e) = cache {
    //         e.insert(load_cell(index, source)?);
    //     }
    //     let res = self.cell.get(&(index, source as u64)).unwrap();
    //     Ok(res)
    // }
    // pub fn load_cell_data(&mut self, index: usize, source: Source) -> Result<&Vec<u8>, SysError> {
    //     let cache = self.data.entry((index, source as u64));
    //     if let Entry::Vacant(e) = cache {
    //         e.insert(load_cell_data(index, source)?);
    //     }
    //     let res = self.data.get(&(index, source as u64)).unwrap();
    //     Ok(res)
    // }

    // fn load_type_hash(&mut self, index: usize, source: Source) -> Result<Option<&[u8;32]>, SysError> {
    //     let cache = self.type_hash.entry((index, source as u64));
    //     if let Entry::Vacant(e) = cache {
    //         e.insert(load_cell_type_hash(index, source)?);
    //     }
    //     let res = self.type_hash.get(&(index, source as u64)).unwrap().as_ref();
    //     Ok(res)
    // }

    // fn load_type(&mut self, index: usize, source: Source) -> Result<Option<&Script>, SysError> {
    //     let cache = self.type_script.entry((index, source as u64));
    //     if let Entry::Vacant(e) = cache {
    //         e.insert(load_cell_type(index, source)?);
    //     }
    //     let res = self.type_script.get(&(index, source as u64)).unwrap().as_ref();
    //     Ok(res)
    // }

    // fn load_lock_hash(&mut self, index: usize, source: Source) -> Result<&[u8;32], SysError> {
    //     let cache = self.lock_hash.entry((index, source as u64));
    //     if let Entry::Vacant(e) = cache {
    //         e.insert(load_cell_lock_hash(index, source)?);
    //     }
    //     let res = self.lock_hash.get(&(index, source as u64)).unwrap();
    //     Ok(res)
    // }

    // fn load_lock_script(&mut self, index: usize, source: Source) -> Result<&Script, SysError> {
    //     let cache = self.lock_script.entry((index, source as u64));
    //     if let Entry::Vacant(e) = cache {
    //         e.insert(load_cell_lock(index, source)?);
    //     }
    //     let res = self.lock_script.get(&(index, source as u64)).unwrap();
    //     Ok(res)
    // }
}

// static WITNESS_PARSER: RefCell<GeneralWitnessParser> = OnceCel  {
//     let mut res = GeneralWitnessParser::default();
//     res.init().unwrap();
//     RefCell::new(res)
// };

pub fn get_witness_parser() -> &'static mut GeneralWitnessParser {
    static mut WITNESS_PARSER: OnceCell<GeneralWitnessParser> = OnceCell::new();
    unsafe {
        WITNESS_PARSER.get_or_init(|| Default::default());
        WITNESS_PARSER.get_mut().unwrap()
    }
}

pub fn get_cell_indexer() -> &'static mut CellIndexer {
    static mut CELL_INDEXER: OnceCell<CellIndexer> = OnceCell::new();
    unsafe {
        CELL_INDEXER.get_or_init(|| Default::default());
        CELL_INDEXER.get_mut().unwrap()
    }
}

impl<T> ParsedWithHash<T> {
    pub fn verify_at(
        &self,
        source: Source,
        index: usize,
        conditions: &[Condition],
    ) -> Result<&T, Box<dyn ScriptError>> {
        for condition in conditions {
            match condition {
                Condition::LockIs(script) => {
                    das_assert!(
                        script.as_slice() == load_cell_lock(index, source)?.as_slice(),
                        ErrorCode::WitnessDataHashOrTypeMissMatch,
                        "Cell {} in {:?} does not have lock {:?}",
                        index,
                        source,
                        script
                    )
                }
                Condition::LockHash(h) => {
                    das_assert!(
                        *h == &load_cell_lock_hash(index, source)?,
                        ErrorCode::WitnessDataHashOrTypeMissMatch,
                        "Cell {} in {:?} does not have lock hash {:?}",
                        index,
                        source,
                        h
                    )
                }
                Condition::TypeIs(script) => {
                    das_assert!(
                        script.as_slice() == load_cell_type(index, source)?.unwrap_or_default().as_slice(),
                        ErrorCode::WitnessDataHashOrTypeMissMatch,
                        "Cell {} in {:?} does not have type {:?}",
                        index,
                        source,
                        script
                    )
                }
                Condition::CodeHashIs(h) => {
                    das_assert!(
                        *h == load_cell_type(index, source)?
                            .unwrap_or_default()
                            .code_hash()
                            .as_slice(),
                        ErrorCode::WitnessDataHashOrTypeMissMatch,
                        "Cell {} in {:?} does not have type id {:?}",
                        index,
                        source,
                        h
                    )
                }
                Condition::TypeHash(h) => {
                    das_assert!(
                        *h == &load_cell_type_hash(index, source)?.unwrap_or_default(),
                        ErrorCode::WitnessDataHashOrTypeMissMatch,
                        "Cell {} in {:?} does not have type hash {:?}",
                        index,
                        source,
                        h
                    )
                }
                Condition::DataIs(d) => {
                    das_assert!(
                        *d == load_cell_data(index, source)?.as_slice(),
                        ErrorCode::WitnessCannotBeVerified,
                        "Data of Cell {} in {:?} does not match the hash({:?}) of the witness",
                        index,
                        source,
                        d
                    )
                }
            }
        }

        Ok(&self.result)
    }

    pub fn verify_unique(&self, source: Source, conditions: &[Condition]) -> Result<&T, Box<dyn ScriptError>> {
        let hash_override = conditions.iter().find(|c| match c {
            Condition::DataIs(_) => true,
            _ => false,
        });

        let hash = match hash_override {
            Some(Condition::DataIs(d)) => <&[u8; 32]>::try_from(*d).unwrap(),
            _ => match &self.hash {
                Some(h) => h,
                None => return Err(code_to_error!(ErrorCode::WitnessCannotBeVerified)),
            },
        };

        let index = get_cell_indexer()
            .find_by_hash(hash, source, CellField::Data)
            .ok_or(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))
            .and_then(|arr| {
                if arr.len() == 1 {
                    Ok(arr[0])
                } else {
                    Err(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))
                }
            })?;
        // let index = match &self.hash {
        //     None => return Err(code_to_error!(ErrorCode::WitnessCannotBeVerified)),
        //     Some(h) => get_cell_indexer().find_by_hash(h, source, CellField::Data),
        // }
        // .ok_or(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))
        // .and_then(|arr| {
        //     if arr.len() == 1 {
        //         Ok(arr[0])
        //     } else {
        //         Err(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))
        //     }
        // })?;
        // let index = cell_found.index;

        self.verify_at(source, index, conditions)
    }

    pub fn verify_any(&self, source: Source, conditions: &[Condition]) -> Result<&T, Box<dyn ScriptError>> {
        let hash_override = conditions.iter().find(|c| match c {
            Condition::DataIs(_) => true,
            _ => false,
        });

        let hash = match hash_override {
            Some(Condition::DataIs(d)) => <&[u8; 32]>::try_from(*d).unwrap(),
            _ => match &self.hash {
                Some(h) => h,
                None => return Err(code_to_error!(ErrorCode::WitnessCannotBeVerified)),
            },
        };

        let indices = get_cell_indexer()
            .find_by_hash(hash, source, CellField::Data)
            .ok_or(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))?;
        if indices.len() == 0 {
            return Err(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch));
        }
        for i in indices {
            match self.verify_at(source, *i, conditions) {
                Ok(res) => return Ok(res),
                Err(_) => continue,
            }
        }
        Err(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))
    }
}

pub struct DataEntity<T> {
    pub index: usize,
    pub version: usize,
    pub entity: T,
}
pub struct EntityWrapper<T> {
    pub old: Option<DataEntity<T>>,
    pub new: Option<DataEntity<T>>,
    pub dep: Option<DataEntity<T>>,
}

pub struct ConfigMap(BTreeMap<String, Bytes>);
impl Deref for ConfigMap {
    type Target = BTreeMap<String, Bytes>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ConfigList> for ConfigMap {
    fn from(value: ConfigList) -> Self {
        Self(
            value
                .into_iter()
                .map(|e| {
                    (
                        String::from_utf8(e.key().as_slice().to_vec()).unwrap(),
                        e.value().raw_data(),
                    )
                })
                .collect(),
        )
    }
}

impl<T> FromWitness for EntityWrapper<T>
where
    T: Entity + 'static,
{
    type Error = Box<dyn ScriptError>;
    fn from_witness(witness: &Witness) -> Result<Self, Box<dyn ScriptError>> {
        if let Witness::Loaded(WithMeta { item, .. }) = witness {
            let data = Data::from_compatible_slice(&item.buf[WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..])
                .map_err(|_| code_to_error!(ErrorCode::WitnessDataDecodingError))?;
            let new = data.new().to_opt().map(|e| DataEntity {
                index: u32::from(e.index()) as usize,
                version: u32::from_le_bytes(e.version().as_slice().try_into().unwrap()) as usize,
                entity: T::from_compatible_slice(e.entity().raw_data().as_slice()).unwrap(),
            });
            let old = data.old().to_opt().map(|e| DataEntity {
                index: u32::from_le_bytes(e.index().as_slice().try_into().unwrap()) as usize,
                version: u32::from_le_bytes(e.version().as_slice().try_into().unwrap()) as usize,
                entity: T::from_compatible_slice(e.entity().raw_data().as_slice()).unwrap(),
            });
            let dep = data.dep().to_opt().map(|e| DataEntity {
                index: u32::from_le_bytes(e.index().as_slice().try_into().unwrap()) as usize,
                version: u32::from_le_bytes(e.version().as_slice().try_into().unwrap()) as usize,
                entity: T::from_compatible_slice(e.entity().raw_data().as_slice()).unwrap(),
            });
            Ok(Self { old, new, dep })
        } else {
            panic!("Witness is still parsing")
        }
    }

    fn parsable(witness: &Witness) -> bool {
        let type_constant = T::get_type_constant() as u32;
        let header_bytes = match witness {
            Witness::Loaded(WithMeta { item, .. }) => {
                &item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
            }
            Witness::Loading(WithMeta { item, .. }) => {
                &item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
            }
        };
        type_constant == u32::from_le_bytes(header_bytes.try_into().unwrap())
    }
}

impl<T> FromWitness for T
where
    T: Entity + 'static,
{
    type Error = Box<dyn ScriptError>;
    default fn from_witness(witness: &Witness) -> Result<Self, Box<dyn ScriptError>> {
        if let Witness::Loaded(WithMeta { item, .. }) = witness {
            Ok(
                T::from_compatible_slice(&item.buf[WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..])
                    .map_err(|_| code_to_error!(ErrorCode::WitnessDataDecodingError))?,
            )
        } else {
            panic!("Witness is still parsing")
        }
    }

    default fn parsable(witness: &Witness) -> bool {
        let type_constant = T::get_type_constant() as u32;
        let header_bytes = match witness {
            Witness::Loaded(WithMeta { item, .. }) => {
                &item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
            }
            Witness::Loading(WithMeta { item, .. }) => {
                &item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
            }
        };
        type_constant == u32::from_le_bytes(header_bytes.try_into().unwrap())
    }
}

impl Witness {
    fn parse<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        let res = match self {
            Witness::Loaded(_) => T::from_witness(self).map_err(|e| e.into())?,
            Witness::Loading(parsing_witness) => {
                let mut buf_vec = vec![0u8; parsing_witness.item.actual_size];
                let loaded_len = parsing_witness.item.buf.len();
                buf_vec[..loaded_len].copy_from_slice(&parsing_witness.item.buf.as_slice());
                load_witness(
                    &mut buf_vec[loaded_len..],
                    loaded_len,
                    parsing_witness.index,
                    Source::Input,
                )?;
                *self = Self::Loaded(WithMeta {
                    item: CompleteWitness {
                        buf: buf_vec,
                        parsed: true,
                    },
                    index: parsing_witness.index,
                    source: parsing_witness.source,
                });
                T::from_witness(self).map_err(|e| e.into())?
            }
        };

        use core::any::Any;
        let hash = (&res as &dyn Any)
            .downcast_ref::<&dyn Blake2BHash>()
            .map(|res| res.blake2b_256());
        Ok(ParsedWithHash { result: res, hash })
    }
}

impl GeneralWitnessParser {
    #[allow(dead_code)]
    fn init(&mut self) -> Result<(), Box<dyn ScriptError>> {
        let mut i = 0;
        let mut buf = [0u8; WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES];
        let mut witnesses: Vec<Witness> = Vec::new();
        loop {
            // Only load first 7 bytes to identify the corresponding witness type
            let res = match load_witness(&mut buf, 0, i, Source::Input) {
                Err(ckb_std::syscalls::SysError::IndexOutOfBound) => break,
                Ok(actual_size) => Witness::Loaded(WithMeta {
                    item: CompleteWitness {
                        buf: buf[..actual_size].to_owned(),
                        parsed: false,
                    },
                    index: i,
                    source: Source::Input,
                }),
                Err(ckb_std::syscalls::SysError::LengthNotEnough(actual_size)) => Witness::Loading(WithMeta {
                    item: PartialWitness {
                        buf: buf.to_vec(),
                        actual_size,
                    },
                    index: i,
                    source: Source::Input,
                }),
                Err(e) => return Err(e.into()),
            };
            witnesses.push(res);
            i += 1;
        }
        self.witnesses = witnesses;
        Ok(())
    }

    pub fn parse_witness<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
        index: usize,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        let res = self.witnesses[index].parse::<T>()?;
        if let Some(hash) = res.hash {
            let _ = self
                .hashes
                .insert(hash, index)
                .is_some_and(|original| panic!("Witness {} and {} have same hash!", index, original));
        }
        Ok(res)
    }

    pub fn find<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
    ) -> Result<Vec<ParsedWithHash<T>>, Box<dyn ScriptError>> {
        let mut res = Vec::new();
        for i in 0..self.witnesses.len() {
            if !T::parsable(&self.witnesses[i]) {
                continue;
            }
            res.push(self.parse_witness(i)?);
        }
        Ok(res)
    }

    pub fn find_unique<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        let mut res = Vec::new();
        for i in 0..self.witnesses.len() {
            if !T::parsable(&self.witnesses[i]) {
                continue;
            }
            res.push(self.parse_witness(i)?);
        }
        das_assert!(
            res.len() == 1,
            ErrorCode::WitnessCannotBeVerified,
            "Multiple witness found"
        );

        Ok(res.pop().unwrap())
    }

    pub fn find_by_hash<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
        hash: &[u8; 32],
    ) -> Result<Option<ParsedWithHash<T>>, Box<dyn ScriptError>> {
        if let Some(index) = self.hashes.get(hash) {
            return self.parse_witness(*index).map(Option::Some);
        }
        for witness in self.witnesses.iter_mut() {
            let parsed = match witness {
                Witness::Loaded(w) => w.item.parsed,
                _ => false,
            };
            if parsed {
                continue;
            }
            if !T::parsable(witness) {
                continue;
            }
            let res = witness.parse::<T>()?;
            match res.hash {
                Some(h) if &h == hash => {
                    return Ok(Some(res));
                }
                _ => continue,
            }
        }

        Ok(None)
    }
}

pub trait GetDataType {
    fn get_type_constant() -> DataType;
}

impl<T> GetDataType for T
where
    T: Entity,
{
    fn get_type_constant() -> DataType {
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
            "DeviceKeyListEntityData" => DataType::DeviceKeyListEntityData,
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
            _ => unreachable!(),
        }
    }
}
