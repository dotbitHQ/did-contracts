use alloc::vec::Vec;
use core::fmt::Debug;
use std::prelude::v1::*;

#[derive(Clone, Debug, Default)]
pub struct Map<K: Debug + PartialEq, V: Clone + Debug + PartialEq> {
    pub items: Vec<(K, V)>,
}

impl<K: Debug + PartialEq, V: Clone + Debug + PartialEq> Map<K, V> {
    pub fn new() -> Self {
        Map { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut index = None;
        for (i, item) in self.items.iter().enumerate() {
            if item.0 == key {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => {
                self.items[i] = (key, value);
            }
            None => {
                self.items.push((key, value));
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> (K, V) {
        let mut index_opt = None;
        for (i, item) in self.items.iter().enumerate() {
            if &item.0 == key {
                index_opt = Some(i);
                break;
            }
        }

        if let Some(index) = index_opt {
            self.items.remove(index)
        } else {
            panic!("removal key (is {:?}) does not exist", key);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        for item in self.items.iter() {
            if &item.0 == key {
                return Some(&item.1);
            }
        }

        None
    }

    pub fn find(&self, value: &V) -> Option<&K> {
        for item in self.items.iter() {
            if &item.1 == value {
                return Some(&item.0);
            }
        }

        None
    }

    pub fn contains(&self, key: &K) -> bool {
        for item in self.items.iter() {
            if &item.0 == key {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let key0 = vec![0u8, 0u8, 0u8];
        let key1 = vec![0u8, 0u8, 1u8];
        let key2 = vec![0u8, 0u8, 2u8];

        let mut map = Map::new();
        map.insert(key0.as_slice(), 0);
        map.insert(key1.as_slice(), 1);
        map.insert(key2.as_slice(), 2);

        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&key0.as_slice()), Some(&0));
        assert_eq!(map.get(&key1.as_slice()), Some(&1));
        assert_eq!(map.get(&key2.as_slice()), Some(&2));
    }

    #[test]
    fn test_remove() {
        let key0 = vec![0u8, 0u8, 0u8];
        let key1 = vec![0u8, 0u8, 1u8];

        let mut map = Map::new();
        map.insert(key0.as_slice(), 0);
        map.insert(key1.as_slice(), 1);

        let ret = map.remove(&key0.as_slice());
        assert_eq!(ret.0, key0.as_slice());
        assert_eq!(ret.1, 0);
    }

    #[test]
    fn test_contains() {
        let key0 = vec![0u8, 0u8, 0u8];
        let key1 = vec![0u8, 0u8, 1u8];
        let key2 = vec![0u8, 0u8, 2u8];

        let mut map = Map::new();
        map.insert(key0.as_slice(), 0);
        map.insert(key1.as_slice(), 1);

        assert_eq!(map.len(), 2);
        assert_eq!(map.contains(&key0.as_slice()), true);
        assert_eq!(map.contains(&key2.as_slice()), false);
    }
}
