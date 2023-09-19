use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::{OnceCell, RefCell};

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{CellOutput, Script};
use ckb_std::high_level::{
    load_cell, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type, load_cell_type_hash, QueryIter,
};
use ckb_std::syscalls::{load_witness, SysError};
use das_types::constants::{DataType, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use molecule::prelude::Entity;

use crate::error::{ErrorCode, ScriptError};
use crate::traits::Blake2BHash;

#[derive(Default)]
pub struct GeneralWitnessParser {
    witnesses: Vec<Witness>,
    hashes: BTreeMap<[u8; 32], usize>,
}

struct PartialWitness {
    buf: Vec<u8>,
    actual_size: usize,
}

struct CompleteWitness {
    buf: Vec<u8>,
    parsed: bool,
}

struct WithMeta<T> {
    item: T,
    index: usize,
    source: Source,
}

enum Witness {
    Loading(WithMeta<PartialWitness>),
    Loaded(WithMeta<CompleteWitness>),
}

trait FromWitness {
    type Error;
    fn from_witness(witness: &Witness) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn parsable(witness: &Witness) -> bool;
}

struct ParsedWithHash<T> {
    result: T,
    hash: Option<[u8; 32]>,
}

#[allow(dead_code)]
enum Condition {
    LockIs(Script),
    TypeIs(Script),
    LockHash([u8; 32]),
    TypeHash([u8; 32]),
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

#[derive(Default)]
pub struct Cached {
    pub data: BTreeMap<(usize, u64), Vec<u8>>,
    // pub type_hash: BTreeMap<(usize, u64), Option<[u8;32]>>,
    // pub type_script: BTreeMap<(usize, u64), Option<Script>>,
    // pub lock_hash: BTreeMap<(usize, u64), [u8;32]>,
    // pub lock_script: BTreeMap<(usize, u64), Script>,
    pub cell: BTreeMap<(usize, u64), CellOutput>,
}
impl Cached {
    pub fn load_cell(&mut self, index: usize, source: Source) -> Result<&CellOutput, SysError> {
        let cache = self.cell.entry((index, source as u64));
        if let Entry::Vacant(e) = cache {
            e.insert(load_cell(index, source)?);
        }
        let res = self.cell.get(&(index, source as u64)).unwrap();
        Ok(res)
    }
    pub fn load_cell_data(&mut self, index: usize, source: Source) -> Result<&Vec<u8>, SysError> {
        let cache = self.data.entry((index, source as u64));
        if let Entry::Vacant(e) = cache {
            e.insert(load_cell_data(index, source)?);
        }
        let res = self.data.get(&(index, source as u64)).unwrap();
        Ok(res)
    }

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

pub fn get_cell_cache() -> &'static mut Cached {
    static mut CELL_CACHE: OnceCell<Cached> = OnceCell::new();
    unsafe {
        CELL_CACHE.get_or_init(|| Default::default());
        CELL_CACHE.get_mut().unwrap()
    }
}

impl<T> ParsedWithHash<T> {
    #[allow(dead_code)]
    fn verify(&self, source: Source, conditions: &[Condition]) -> Result<&T, Box<dyn ScriptError>> {
        let cell_found = match &self.hash {
            None => return Err(code_to_error!(ErrorCode::WitnessCannotBeVerified)),
            Some(h) => QueryIter::new(
                |index, source| {
                    let res = get_cell_cache().load_cell_data(index, source)?;
                    Ok(WithMeta {
                        item: res,
                        index,
                        source,
                    })
                },
                source,
            )
            .find(|WithMeta { item, .. }| *item.as_slice() == h[..]),
        }
        .ok_or(code_to_error!(ErrorCode::WitnessDataHashOrTypeMissMatch))?;
        let index = cell_found.index;
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
                        *h == load_cell_lock_hash(index, source)?,
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
                Condition::TypeHash(h) => {
                    das_assert!(
                        *h == load_cell_type_hash(index, source)?.unwrap_or_default(),
                        ErrorCode::WitnessDataHashOrTypeMissMatch,
                        "Cell {} in {:?} does not have type hash {:?}",
                        index,
                        source,
                        h
                    )
                }
            }
        }
        Ok(&self.result)
    }
}

impl<T> FromWitness for T
where
    T: Entity + 'static,
{
    type Error = Box<dyn ScriptError>;
    fn from_witness(witness: &Witness) -> Result<Self, Box<dyn ScriptError>> {
        if let Witness::Loaded(WithMeta { item, .. }) = witness {
            // let type_constant = T::get_type_constant();
            // das_assert!(
            //     Self::parsable(witness),
            //     ErrorCode::WitnessDataDecodingError,
            //     "The data type constant: {:?} and the actual molecule structure: {} does not match",
            //     type_constant,
            //     T::NAME
            // );
            Ok(
                T::from_compatible_slice(&item.buf[WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..])
                    .map_err(|_| code_to_error!(ErrorCode::WitnessDataDecodingError))?,
            )
        } else {
            panic!("Witness is still parsing")
        }
    }

    fn parsable(witness: &Witness) -> bool {
        let type_constant = T::get_type_constant() as u32;
        match witness {
            Witness::Loaded(WithMeta { item, .. }) => {
                type_constant
                    == u32::from_be_bytes(
                        item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
                            .try_into()
                            .unwrap(),
                    )
            }
            Witness::Loading(WithMeta { item, .. }) => {
                type_constant
                    == u32::from_be_bytes(
                        item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
                            .try_into()
                            .unwrap(),
                    )
            }
        }
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

    #[allow(dead_code)]
    fn parse_witness<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
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

    #[allow(dead_code)]
    fn find<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
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

    #[allow(dead_code)]
    fn find_by_hash<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
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
        }
    }
}
