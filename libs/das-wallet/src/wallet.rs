use alloc::vec::Vec;
use ckb_std::debug;
use std::prelude::v1::*;

#[derive(Debug)]
pub struct Wallet {
    pub items: Vec<(Vec<u8>, u64)>,
}

impl Wallet {
    pub fn new() -> Self {
        Wallet { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn add_balance(&mut self, wallet_id: &[u8], balance: u64) {
        // If wallet exists, add balance to it.
        for item in self.items.iter_mut() {
            if item.0.as_slice() == wallet_id {
                item.1 += balance;
                return;
            }
        }

        // If Wallet does not exist, create it.
        self.items.push((wallet_id.to_vec(), balance));
    }

    pub fn get_balance(&mut self, wallet_id: &[u8]) -> Option<u64> {
        for item in self.items.iter() {
            if item.0.as_slice() == wallet_id {
                return Some(item.1);
            }
        }

        None
    }

    pub fn cmp_balance(&self, wallet_id: &[u8], balance: u64) -> Result<bool, ()> {
        for item in self.items.iter() {
            if item.0.as_slice() == wallet_id {
                return Ok(item.1 == balance);
            }
        }

        debug!(
            "Wallet {:?} not found, current wallets: {:?}",
            wallet_id, self.items
        );
        Err(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wallet_basic_function() {
        let wallet_1 = [1, 0, 0];
        let wallet_2 = [2, 0, 0];

        let mut wallet = Wallet::new();
        wallet.add_balance(&wallet_1, 100);
        wallet.add_balance(&wallet_1, 100);
        wallet.add_balance(&wallet_2, 100);

        assert_eq!(wallet.get_balance(&wallet_1), Some(200));

        assert!(wallet.cmp_balance(&wallet_1, 200).unwrap());
        assert!(wallet.cmp_balance(&wallet_2, 100).unwrap());
    }

    #[test]
    fn test_wallet_compare_with_not_exists() {
        let wallet_1 = [1, 0, 0];
        let wallet_2 = [2, 0, 0];

        let mut wallet = Wallet::new();
        wallet.add_balance(&wallet_1, 100);

        assert!(wallet.cmp_balance(&wallet_2, 200).is_err());
    }
}
