use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::*;
use ckb_std::ckb_types::prelude::*;
use ckb_std::error::SysError;
use ckb_std::{high_level, syscalls};
use das_types::constants::DasLockType;

use super::error::*;
use super::{code_to_error, util};
use crate::constants::{ScriptType, CKB_HASH_DIGEST, EIP712_CHAINID_SIZE, SECP_SIGNATURE_SIZE};

fn find_input_size() -> Result<usize, Box<dyn ScriptError>> {
    let mut i = 1;
    loop {
        let mut buf = [0u8; 1];
        match syscalls::load_input(&mut buf, 0, i, Source::Input) {
            Ok(_) => {
                // continue counting ...
            }
            Err(SysError::LengthNotEnough(_)) => {
                // continue counting ...
            }
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(i)
}

pub fn calc_digest_by_lock(
    sign_type: DasLockType,
    script: ScriptReader,
) -> Result<([u8; 32], Vec<u8>), Box<dyn ScriptError>> {
    let input_group_idxs = util::find_cells_by_script(ScriptType::Lock, script, Source::Input)?;
    let ret = calc_digest_by_input_group(sign_type, input_group_idxs)?;

    Ok(ret)
}

pub fn get_eip712_digest(
    input_group_idxs: Vec<usize>,
) -> Result<([u8; 32], [u8; 32], Vec<u8>, Vec<u8>), Box<dyn ScriptError>> {
    let init_witness_idx = input_group_idxs[0];
    let (digest, witness_args_lock) = calc_digest_by_input_group(DasLockType::ETHTypedData, input_group_idxs)?;

    das_assert!(
        witness_args_lock.len() == SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST + EIP712_CHAINID_SIZE,
        ErrorCode::EIP712SignatureError,
        "Inputs[{}] The length of signature is invalid.(current: {}, expected: {})",
        init_witness_idx,
        witness_args_lock.len(),
        SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST + EIP712_CHAINID_SIZE
    );

    let from = SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST;
    let to = from + EIP712_CHAINID_SIZE;
    let eip712_chain_id = witness_args_lock[from..to].to_vec();

    let mut typed_data_hash = [0u8; 32];
    typed_data_hash.copy_from_slice(&witness_args_lock[SECP_SIGNATURE_SIZE..SECP_SIGNATURE_SIZE + CKB_HASH_DIGEST]);

    Ok((digest, typed_data_hash, eip712_chain_id, witness_args_lock))
}

pub fn calc_digest_by_input_group(
    sign_type: DasLockType,
    input_group_idxs: Vec<usize>,
) -> Result<([u8; 32], Vec<u8>), Box<dyn ScriptError>> {
    debug!(
        "Calculate digest by input group ... (sign_type: {:?}, input_group: {:?})",
        sign_type, input_group_idxs
    );

    let init_witness_idx = input_group_idxs[0];
    let witness_bytes = util::load_witnesses(init_witness_idx)?;
    let init_witness = WitnessArgs::from_slice(&witness_bytes).map_err(|_| {
        warn!(
            "  inputs[{}] Witness can not be decoded as WitnessArgs.(data: 0x{})",
            init_witness_idx,
            util::hex_string(&witness_bytes)
        );
        ErrorCode::WitnessArgsDecodingError
    })?;

    // Extract signatures and reset witness_args to empty status for calculation of digest.
    match init_witness.as_reader().lock().to_opt() {
        None => Err(code_to_error!(ErrorCode::WitnessArgsInvalid)),
        Some(witness_args_lock) => {
            debug!(
                "  inputs[{}] Generating digest ... (sign_type: {:?}, witness_args.lock: 0x{}",
                init_witness_idx,
                sign_type,
                util::first_n_bytes_to_hex(witness_args_lock.raw_data(), 20)
            );

            let empty_lock_bytes = match sign_type {
                DasLockType::CKBMulti => {
                    let bytes = witness_args_lock.raw_data();
                    let _reserved_byte = bytes[0];
                    let _require_first_n = bytes[1];
                    let threshold = bytes[2] as usize;
                    let signature_addresses_len = bytes[3];
                    let slice_point = (4 + 20 * signature_addresses_len) as usize;

                    let _signatures = bytes[slice_point..].to_vec();
                    debug!(
                        "  inputs[{}] Slice WitnessArgs.lock at {} .(header: 0x{}, args: 0x{}, signatures: {})",
                        init_witness_idx,
                        slice_point,
                        util::hex_string(&bytes[..4]),
                        util::hex_string(&bytes[4..slice_point]),
                        util::first_n_bytes_to_hex(&_signatures, 10)
                    );

                    let mut data: Vec<u8> = bytes[..slice_point].to_vec();
                    data.extend_from_slice(&vec![0u8; SECP_SIGNATURE_SIZE * threshold]);

                    data
                }
                _ => {
                    vec![0u8; witness_args_lock.len()]
                }
            };
            let empty_signature = BytesOpt::new_builder().set(Some(empty_lock_bytes.pack())).build();
            let empty_witness = init_witness.clone().as_builder().lock(empty_signature).build();

            let tx_hash = high_level::load_tx_hash().map_err(|_| ErrorCode::ItemMissing)?;

            debug!(
                "  inputs[{}] calculating digest, concat tx_hash: 0x{}, concat witness_args.len(): 0x{} witness_args: 0x{}",
                init_witness_idx,
                util::hex_string(&tx_hash),
                util::hex_string(&(empty_witness.as_bytes().len() as u64).to_le_bytes()),
                util::hex_string(&empty_witness.as_bytes())
            );

            let mut blake2b = util::new_blake2b();
            blake2b.update(&tx_hash);
            blake2b.update(&(empty_witness.as_bytes().len() as u64).to_le_bytes());
            blake2b.update(&empty_witness.as_bytes());
            for idx in input_group_idxs.iter().skip(1).cloned() {
                let other_witness_bytes = util::load_witnesses(idx)?;
                blake2b.update(&(other_witness_bytes.len() as u64).to_le_bytes());
                blake2b.update(&other_witness_bytes);
                debug!(
                    "  inputs[{}] calculating digest, concat witness[{}].len(): 0x{}, witness[{}]: 0x{}",
                    init_witness_idx,
                    idx,
                    util::hex_string(&(other_witness_bytes.len() as u64).to_le_bytes()),
                    idx,
                    util::hex_string(&other_witness_bytes)
                );
            }
            let mut i = find_input_size()?;
            loop {
                let ret = util::load_witnesses(i);
                match ret {
                    Ok(outter_witness_bytes) => {
                        blake2b.update(&(outter_witness_bytes.len() as u64).to_le_bytes());
                        blake2b.update(&outter_witness_bytes);

                        debug!(
                            "  inputs[{}] calculating digest, concat outter_witness[{}].len(): 0x{}, outter_witness[{}]: 0x{}",
                            init_witness_idx,
                            i,
                            util::hex_string(&(outter_witness_bytes.len() as u64).to_le_bytes()),
                            i,
                            util::hex_string(&outter_witness_bytes)
                        );
                    }
                    Err(err) => {
                        if err.as_i8() == ErrorCode::IndexOutOfBound as i8 {
                            break;
                        } else {
                            return Err(err);
                        }
                    }
                }

                i += 1;
            }
            let mut digest = [0u8; 32];
            blake2b.finalize(&mut digest);

            debug!(
                "  inputs[{}] Generate digest: 0x{}",
                init_witness_idx,
                util::hex_string(&digest)
            );

            Ok((digest, witness_args_lock.raw_data().to_vec()))
        }
    }
}
