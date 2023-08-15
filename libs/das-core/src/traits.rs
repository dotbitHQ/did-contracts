use das_types::prelude::Entity;

use crate::util;

pub trait Blake2BHash {
    fn blake2b_256(&self) -> [u8; 32];
}

impl<T> Blake2BHash for T
where
    T: Entity,
{
    fn blake2b_256(&self) -> [u8; 32] {
        util::blake2b_256(self.as_slice())
    }
}
