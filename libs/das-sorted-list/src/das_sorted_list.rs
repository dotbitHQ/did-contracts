use super::util::cmp;
use alloc::vec::Vec;
use std::prelude::v1::*;

#[derive(Debug)]
pub struct DasSortedList {
    items: Vec<Vec<u8>>,
}

impl DasSortedList {
    pub fn new(mut items: Vec<Vec<u8>>) -> Self {
        if !items.is_empty() {
            items.sort_by(|a, b| cmp(a, b));
        }

        DasSortedList { items }
    }

    pub fn items(&self) -> &[Vec<u8>] {
        &self.items
    }

    pub fn cmp_order_with(&self, targets: &[Vec<u8>]) -> bool {
        for (index, item) in self.items.iter().enumerate() {
            if item != &targets[index] {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::super::util::hex_to_bytes;
    use super::*;
    use alloc::vec;

    #[test]
    fn test_sorted_list_cmp() {
        let raw: Vec<&str> = vec![
            "0x1000", "0x2000", "0x1100", "0x1200", "0xa000", "0xb000", "0xa100", "0xb100", "0x1234", "0x0000",
            "0x0001",
        ];
        let data = raw.into_iter().map(|item| hex_to_bytes(item)).collect();

        let sorted_list = DasSortedList::new(data);

        let expected_raw: Vec<&str> = vec![
            "0x0000", "0x0001", "0x1000", "0x1100", "0x1200", "0x1234", "0x2000", "0xa000", "0xa100", "0xb000",
            "0xb100",
        ];
        let expected_data: Vec<Vec<u8>> = expected_raw.into_iter().map(|item| hex_to_bytes(item)).collect();

        assert!(sorted_list.cmp_order_with(&expected_data));
    }
}
