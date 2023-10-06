use das_types::packed::DataEntity;
use das_types::prelude::Entity;

use crate::util;

pub trait Blake2BHash {
    fn blake2b_256(&self) -> [u8; 32];
}

impl<T: Entity> Blake2BHash for T {
    default fn blake2b_256(&self) -> [u8; 32] {
        util::blake2b_256(self.as_slice())
    }
}

impl Blake2BHash for DataEntity {
    fn blake2b_256(&self) -> [u8; 32] {
        util::blake2b_256(self.entity().as_reader().raw_data())
    }
}
