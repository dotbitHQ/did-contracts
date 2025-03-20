#![cfg_attr(target_arch = "riscv64", no_std)]

extern crate alloc;
use alloc::vec::Vec;

use ckb_std::debug;
use error::ErrorCode;
pub mod error;

#[derive(Debug)]
pub struct DidCellAccountInfo {
    pub account: Vec<u8>,
    pub expire_at: u64,
    pub hash: [u8; 20],
}

/// data: [0u8][version; u8|1byte][witness_hash; 20bytes][expire_at: 8bytes|uint64][account; n bytes]
pub fn parse_did_cell_account_info(data: &[u8]) -> Result<DidCellAccountInfo, ErrorCode> {
    const PREFIX_SIZE: usize = 1;
    const VER_SIZE: usize = 1;
    const WITNESS_HASH_SIZE: usize = 20;
    const EXPIRE_AT_SIZE: usize = 8;
    if data.len() <= PREFIX_SIZE + VER_SIZE + WITNESS_HASH_SIZE + EXPIRE_AT_SIZE {
        debug!("did-cell account info len invalid");
        return Err(ErrorCode::Encoding);
    }
    // remove prefix byte
    let data = &data[PREFIX_SIZE..];
    match data[0] {
        // version 1
        1u8 => {
            let mut witness_hash = [0u8; WITNESS_HASH_SIZE];
            witness_hash.clone_from_slice(&data[VER_SIZE..VER_SIZE + WITNESS_HASH_SIZE]);

            let mut buf = [0u8; EXPIRE_AT_SIZE];
            buf.copy_from_slice(&data[VER_SIZE + WITNESS_HASH_SIZE..VER_SIZE + WITNESS_HASH_SIZE + EXPIRE_AT_SIZE]);
            let expire_at = u64::from_le_bytes(buf);

            let account: Vec<u8> = (&data[VER_SIZE + WITNESS_HASH_SIZE + EXPIRE_AT_SIZE..]).to_vec();

            return Ok(DidCellAccountInfo {
                account,
                expire_at,
                hash: witness_hash,
            });
        }
        _ => {
            debug!("unknown did-cell account info version");
            return Err(ErrorCode::Encoding);
        }
    }
}
