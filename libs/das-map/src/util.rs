use super::map::Map;
use core::fmt::Debug;

pub fn add<K, V>(map: &mut Map<K, V>, key: K, value: V)
where
    K: Clone + Debug + PartialEq,
    V: Clone + Debug + core::ops::Add<Output = V>,
{
    let exist_value = map.get(&key);
    if exist_value.is_some() {
        let new_value = exist_value.map(|v| v.clone()).unwrap() + value;
        map.insert(key.clone(), new_value);
    } else {
        map.insert(key.clone(), value);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_add() {
        let key0 = vec![0u8, 0u8, 0u8];
        let mut map = Map::new();
        map.insert(key0.as_slice(), 0);

        assert_eq!(map.get(&key0.as_slice()), Some(&0));

        add(&mut map, &key0.as_slice(), 100);

        assert_eq!(map.get(&key0.as_slice()), Some(&100));
    }
}
