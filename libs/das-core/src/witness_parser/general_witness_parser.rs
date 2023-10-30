use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::OnceCell;
use core::marker::PhantomData;
use core::ops::Deref;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{CellOutput, Script};
use ckb_std::high_level::{
    load_cell, load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type, load_cell_type_hash, QueryIter,
};
use ckb_std::syscalls::{load_witness, SysError};
use das_types::constants::{WITNESS_HEADER, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use das_types::packed::{ConfigList, Data, DataEntity};
use molecule::bytes::Bytes;
use molecule::prelude::Entity;

use crate::error::{ErrorCode, ScriptError};
use crate::traits::{Blake2BHash, GetDataType};
// use crate::util::find_only_cell_by_type_id;

#[derive(Default, Clone, Debug)]
pub struct GeneralWitnessParser {
    witnesses: Vec<Witness>,
    hashes: BTreeMap<[u8; 32], usize>,
}

#[derive(Clone, Debug)]
pub struct PartialWitness {
    pub buf: Vec<u8>,
    pub actual_size: usize,
}

#[derive(Clone, Debug)]
pub struct CompleteWitness {
    pub buf: Vec<u8>,
    pub parsed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Meta {
    pub index: usize,
    pub source: Source,
}

#[derive(Clone, Debug)]
pub struct WithMeta<T> {
    pub item: T,
    pub meta: Meta,
}

impl<T> WithMeta<T> {
    pub fn new(item: T, meta: Meta) -> Self {
        Self { item, meta }
    }

    pub fn get_meta(&self) -> &Meta {
        &self.meta
    }

    pub fn get_item(&self) -> &T {
        &self.item
    }
}

impl<T> Deref for WithMeta<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[derive(Clone, Debug)]
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

    fn hash(&self) -> Option<[u8; 32]> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct ParsedWithHash<T> {
    pub result: T,
    pub hash: Option<[u8; 32]>,
}

#[derive(Clone, Debug)]
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

#[derive(Default, Debug)]
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
                meta: Meta { index, source },
            })
        };
        let mut index_fn = |item: WithMeta<CellOutput>| {
            self.by_hash
                .entry((
                    item.item.lock().code_hash().as_slice().try_into().unwrap(),
                    item.meta.source as u64,
                    CellField::Lock,
                ))
                .and_modify(|v| v.push(item.meta.index))
                .or_insert(vec![item.meta.index]);

            item.item.type_().to_opt().map(|script| {
                self.by_hash
                    .entry((
                        script.code_hash().as_slice().try_into().unwrap(),
                        item.meta.source as u64,
                        CellField::Type,
                    ))
                    .and_modify(|v| v.push(item.meta.index))
                    .or_insert(vec![item.meta.index])
            });

            let mut data = [0; 32];
            let res = ckb_std::syscalls::load_cell_data(&mut data, 0, item.meta.index, item.meta.source);
            match res {
                Ok(_) | Err(SysError::LengthNotEnough(_)) => {
                    self.by_hash
                        .entry((data, item.meta.source as u64, CellField::Data))
                        .and_modify(|v| v.push(item.meta.index))
                        .or_insert(vec![item.meta.index]);
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
        WITNESS_PARSER.get_or_init(|| {
            let mut res = GeneralWitnessParser::default();
            res.init().unwrap();
            res
        });
        WITNESS_PARSER.get_mut().unwrap()
    }
}

pub fn get_cell_indexer() -> &'static mut CellIndexer {
    static mut CELL_INDEXER: OnceCell<CellIndexer> = OnceCell::new();
    unsafe {
        CELL_INDEXER.get_or_init(|| {
            let mut res = CellIndexer::default();
            res.init().unwrap();
            res
        });
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

// #[derive(Clone, Debug)]
// pub struct DataEntity<T> {
//     pub index: usize,
//     pub version: usize,
//     pub entity: T,
// }
#[derive(Clone, Debug)]
pub struct EntityWrapper<const T: u32, H = ForDefault> {
    pub inner: Data,
    _marker: PhantomData<H>,
}

#[derive(Clone, Debug)]
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

pub struct ForOld();
pub struct ForNew();
pub struct ForDep();
pub struct ForDefault();

impl<const T: u32, A> EntityWrapper<T, A> {
    pub fn into<B>(self) -> EntityWrapper<T, B> {
        EntityWrapper {
            inner: self.inner,
            _marker: Default::default(),
        }
    }

    pub fn from<B>(value: EntityWrapper<T, B>) -> Self {
        Self {
            inner: value.inner,
            _marker: Default::default(),
        }
    }
}

impl<const T: u32> EntityWrapper<T, ForOld> {
    pub fn into_target(self) -> Option<DataEntity> {
        self.inner.old().to_opt()
    }
}

impl<const T: u32> EntityWrapper<T, ForNew> {
    pub fn into_target(self) -> Option<DataEntity> {
        self.inner.new().to_opt()
    }
}

impl<const T: u32> EntityWrapper<T, ForDep> {
    pub fn into_target(self) -> Option<DataEntity> {
        self.inner.dep().to_opt()
    }
}

trait GenHash {
    fn gen_hash<const T: u32, H>(e: &EntityWrapper<T, H>) -> Option<[u8; 32]>;
}

impl GenHash for ForOld {
    fn gen_hash<const T: u32, H>(e: &EntityWrapper<T, H>) -> Option<[u8; 32]> {
        e.inner.old().to_opt().as_ref().map(|i| i.blake2b_256())
    }
}

impl GenHash for ForNew {
    fn gen_hash<const T: u32, H>(e: &EntityWrapper<T, H>) -> Option<[u8; 32]> {
        e.inner.new().to_opt().as_ref().map(|i| i.blake2b_256())
    }
}

impl GenHash for ForDep {
    fn gen_hash<const T: u32, H>(e: &EntityWrapper<T, H>) -> Option<[u8; 32]> {
        e.inner.dep().to_opt().as_ref().map(|i| i.blake2b_256())
    }
}

impl GenHash for ForDefault {
    fn gen_hash<const T: u32, H>(_e: &EntityWrapper<T, H>) -> Option<[u8; 32]> {
        None
    }
}

impl<const T: u32, H: GenHash> FromWitness for EntityWrapper<T, H>
where
    [u8; T as usize]: Sized,
{
    type Error = Box<dyn ScriptError>;
    fn from_witness(witness: &Witness) -> Result<Self, Box<dyn ScriptError>> {
        if let Witness::Loaded(WithMeta { item, .. }) = witness {
            // let data = Data::from_compatible_slice(&item.buf[WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..])
            //     .map_err(|_| code_to_error!(ErrorCode::WitnessDataDecodingError))?;
            // let new = data.new().to_opt().map(|e| DataEntity {
            //     index: u32::from(e.index()) as usize,
            //     version: u32::from_le_bytes(e.version().as_slice().try_into().unwrap()) as usize,
            //     entity: T::from_compatible_slice(e.entity().raw_data().as_slice()).unwrap(),
            // });
            // let old = data.old().to_opt().map(|e| DataEntity {
            //     index: u32::from_le_bytes(e.index().as_slice().try_into().unwrap()) as usize,
            //     version: u32::from_le_bytes(e.version().as_slice().try_into().unwrap()) as usize,
            //     entity: T::from_compatible_slice(e.entity().raw_data().as_slice()).unwrap(),
            // });
            // let dep = data.dep().to_opt().map(|e| DataEntity {
            //     index: u32::from_le_bytes(e.index().as_slice().try_into().unwrap()) as usize,
            //     version: u32::from_le_bytes(e.version().as_slice().try_into().unwrap()) as usize,
            //     entity: T::from_compatible_slice(e.entity().raw_data().as_slice()).unwrap(),
            // });
            let inner = Data::from_compatible_slice(&item.buf[WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..])
                .map_err(|_| code_to_error!(ErrorCode::WitnessDataDecodingError))?;
            Ok(Self {
                inner,
                _marker: Default::default(),
            })
        } else {
            panic!("Witness is still parsing")
        }
    }

    fn parsable(witness: &Witness) -> bool {
        let type_constant = [0; T as usize].len() as u32;
        let header_bytes = match witness {
            Witness::Loaded(WithMeta { item, .. }) => &item.buf[0..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES],
            Witness::Loading(WithMeta { item, .. }) => &item.buf[0..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES],
        };
        &header_bytes[0..WITNESS_HEADER_BYTES] == &WITNESS_HEADER
            && type_constant == u32::from_le_bytes(header_bytes[WITNESS_HEADER_BYTES..].try_into().unwrap())
    }

    fn hash(&self) -> Option<[u8; 32]> {
        H::gen_hash(self)
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
            Witness::Loaded(WithMeta { item, .. }) => &item.buf[0..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES],
            Witness::Loading(WithMeta { item, .. }) => &item.buf[0..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES],
        };
        &header_bytes[0..WITNESS_HEADER_BYTES] == &WITNESS_HEADER
            && type_constant == u32::from_le_bytes(header_bytes[WITNESS_HEADER_BYTES..].try_into().unwrap())
    }

    default fn hash(&self) -> Option<[u8; 32]> {
        Some(self.blake2b_256())
    }
}

impl Witness {
    fn get_meta(&self) -> Meta {
        match self {
            Witness::Loaded(w) => w.meta,
            Witness::Loading(w) => w.meta,
        }
    }
    fn load_complete(&mut self) -> Result<(), Box<dyn ScriptError>> {
        match self {
            Witness::Loading(parsing_witness) => {
                let mut buf_vec = vec![0u8; parsing_witness.item.actual_size];
                let loaded_len = parsing_witness.item.buf.len();
                buf_vec[..loaded_len].copy_from_slice(&parsing_witness.item.buf.as_slice());
                load_witness(
                    &mut buf_vec[loaded_len..],
                    loaded_len,
                    parsing_witness.meta.index,
                    Source::Input,
                )?;
                *self = Self::Loaded(WithMeta {
                    item: CompleteWitness {
                        buf: buf_vec,
                        parsed: true,
                    },
                    meta: parsing_witness.meta,
                });
            }
            _ => (),
        };

        Ok(())
    }

    fn parse<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        self.load_complete()?;
        // let res = match self {
        //     Witness::Loaded(_) => T::from_witness(self).map_err(|e| e.into())?,
        //     Witness::Loading(parsing_witness) => {
        //         // let mut buf_vec = vec![0u8; parsing_witness.item.actual_size];
        //         // let loaded_len = parsing_witness.item.buf.len();
        //         // buf_vec[..loaded_len].copy_from_slice(&parsing_witness.item.buf.as_slice());
        //         // load_witness(
        //         //     &mut buf_vec[loaded_len..],
        //         //     loaded_len,
        //         //     parsing_witness.meta.index,
        //         //     Source::Input,
        //         // )?;
        //         // *self = Self::Loaded(WithMeta {
        //         //     item: CompleteWitness {
        //         //         buf: buf_vec,
        //         //         parsed: true,
        //         //     },
        //         //     meta: parsing_witness.meta,
        //         // });
        //         T::from_witness(self).map_err(|e| e.into())?
        //     }
        // };
        let res = T::from_witness(self).map_err(|e| e.into())?;
        let hash = res.hash();
        // use core::any::Any;
        // let hash = (&res as &dyn Any)
        //     .downcast_ref::<&dyn Blake2BHash>()
        //     .map(|res| res.blake2b_256());
        Ok(ParsedWithHash { result: res, hash })
    }
}

impl GeneralWitnessParser {
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
                    meta: Meta {
                        index: i,
                        source: Source::Input,
                    },
                }),
                Err(ckb_std::syscalls::SysError::LengthNotEnough(actual_size)) => Witness::Loading(WithMeta {
                    item: PartialWitness {
                        buf: buf.to_vec(),
                        actual_size,
                    },
                    meta: Meta {
                        index: i,
                        source: Source::Input,
                    },
                }),
                Err(e) => return Err(e.into()),
            };
            witnesses.push(res);
            i += 1;
        }
        self.witnesses = witnesses;
        Ok(())
    }

    pub fn get_das_witness(&mut self, index: usize) -> Result<&WithMeta<CompleteWitness>, Box<dyn ScriptError>> {
        let res = self
            .witnesses
            .iter_mut()
            .filter(|w| match w {
                Witness::Loaded(w) => w.buf.starts_with(&WITNESS_HEADER[..]),
                Witness::Loading(w) => w.buf.starts_with(&WITNESS_HEADER[..]),
            })
            .nth(index);

        match res {
            Some(w) => {
                w.load_complete()?;
                match w {
                    Witness::Loaded(res) => Ok(res),
                    _ => unreachable!(),
                }
            }
            None => Err(code_to_error!(ErrorCode::IndexOutOfBound)),
        }
    }

    // TODO: should support DepGroup
    pub fn parse_witness<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
        index: usize,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        let res = self.witnesses[index].parse::<T>()?;
        if let Some(hash) = res.hash {
            let _ = self.hashes.insert(hash, index).is_some_and(|original| {
                if original != index {
                    panic!("Witness {} and {} have same hash!", index, original)
                } else {
                    false
                }
            });
        }
        Ok(res)
    }

    pub fn parse_for_cell<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &mut self,
        meta: &Meta,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        let mut data = [0; 32];
        let res = ckb_std::syscalls::load_cell_data(&mut data, 0, meta.index, meta.source)?;
        if res != 32 {
            return Err(code_to_error!(ErrorCode::InvalidCellData));
        }

        self.find_by_hash(&data)
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
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        if let Some(index) = self.hashes.get(hash) {
            return self.parse_witness(*index);
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
                    self.hashes.insert(h, witness.get_meta().index);
                    return Ok(res);
                }
                _ => continue,
            }
        }

        Err(code_to_error!(ErrorCode::WitnessCannotBeVerified))
    }
}
