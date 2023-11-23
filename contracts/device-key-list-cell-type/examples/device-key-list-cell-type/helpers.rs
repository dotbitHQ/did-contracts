use ckb_std::ckb_types::packed::Uint64;
use das_types::constants::DataType;
use molecule::prelude::Entity;

pub trait GetDataType {
    fn get_type_constant() -> DataType;
}

impl<T> GetDataType for T
where
    T: Entity,
{
    fn get_type_constant() -> DataType {
        match T::NAME {
            "DeviceKeyListCellData" => DataType::DeviceKeyListEntityData,
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
