use core::ops::Deref;

use ckb_std::ckb_types::packed::Uint64;
use das_types::constants::DataType;
use molecule::prelude::Entity;

pub struct Comparable<T>(pub T);

impl<T> Deref for Comparable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialOrd for Comparable<T>
where
    T: Entity,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T> Ord for Comparable<T>
where
    T: Entity,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T> PartialEq for Comparable<T>
where
    T: Entity,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T> Eq for Comparable<T> where T: Entity {}

impl<T> Clone for Comparable<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
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
            "DeviceKeyListCellData" => DataType::DeviceKeyList,
            _ => unimplemented!(),
        }
    }
}

pub trait ToNum {
    type Target;
    const BYTE: usize;
    fn to_num(self) -> Self::Target;
}

impl ToNum for Uint64 {
    type Target = u64;
    const BYTE: usize = 8;

    fn to_num(self) -> Self::Target {
        let mut buf = [0u8; Self::BYTE];
        buf.copy_from_slice(self.as_slice());
        Self::Target::from_le_bytes(buf)
    }
}
