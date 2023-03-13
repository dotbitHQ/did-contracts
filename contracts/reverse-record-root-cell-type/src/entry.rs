use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::CellField;
use das_core::error::*;
use das_core::witness_parser::reverse_record::{ReverseRecordWitness, ReverseRecordWitnessesParser};
use das_core::witness_parser::WitnessesParser;
use das_core::{assert as das_assert, code_to_error, debug, util, verifiers, warn};
use das_dynamic_libs::constants::DynLibName;
use das_dynamic_libs::error::Error as DasDynamicLibError;
use das_dynamic_libs::sign_lib::SignLib;
use das_dynamic_libs::{load_2_methods, load_lib, log_loading, new_context};
use das_types::constants::DasLockType;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running reverse-record-root-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    match action {
        b"create_reverse_record_root" => {
            util::require_super_lock()?;

            parser.parse_cell()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            verifiers::common::verify_cell_number_and_position(
                "ReverseRecordRootCell",
                &input_cells,
                &[],
                &output_cells,
                &[0],
            )?;

            debug!("Verify all fields of the new ReverseRecordRootCell.");

            // verify capacity
            let root_cell_capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;
            let expected_capacity = u64::from(config_reverse_resolution.record_basic_capacity());

            das_assert!(
                root_cell_capacity == expected_capacity,
                ReverseRecordRootCellErrorCode::InitialCapacityError,
                "The initial capacity of ReverseRecordRootCell should be equal to ConfigCellReverseResolution.record_basic_capacity .(expected: {}, current: {})",
                expected_capacity,
                root_cell_capacity
            );

            // verify lock
            verifiers::misc::verify_always_success_lock(output_cells[0], Source::Output)?;

            // verify data
            let output_data = util::load_cell_data(output_cells[0], Source::Output)?;
            das_assert!(
                output_data == vec![0u8; 32],
                ReverseRecordRootCellErrorCode::InitialOutputsDataError,
                "The initial outputs_data of ReverseRecordRootCell should be 32 bytes of 0x00."
            );
        }
        b"update_reverse_record_root" => {
            let config_smt_white_list = parser.configs.smt_node_white_list()?;
            verify_has_some_lock_in_white_list(1, config_smt_white_list)?;

            let _config_reverse = parser.configs.reverse_resolution()?;

            verifiers::common::verify_cell_number_and_position(
                "ReverseRecordRootCell",
                &input_cells,
                &[0],
                &output_cells,
                &[0],
            )?;

            verifiers::common::verify_cell_consistent_with_exception(
                "ReverseRecordRootCell",
                input_cells[0],
                output_cells[0],
                vec![CellField::Data],
            )?;

            let mut sign_lib = SignLib::new();
            // ⚠️ This must be exist in the top level, because we need to use the libraries later.
            let mut eth_context = new_context!();
            let mut tron_context = new_context!();
            let mut doge_context = new_context!();
            if cfg!(not(feature = "dev")) {
                log_loading!(DynLibName::ETH);
                let eth_lib = load_lib!(eth_context, DynLibName::ETH);
                sign_lib.eth = load_2_methods!(eth_lib);

                log_loading!(DynLibName::TRON);
                let tron_lib = load_lib!(tron_context, DynLibName::TRON);
                sign_lib.tron = load_2_methods!(tron_lib);

                log_loading!(DynLibName::DOGE);
                let doge_lib = load_lib!(doge_context, DynLibName::DOGE);
                sign_lib.doge = load_2_methods!(doge_lib);
            }

            debug!("Start iterating ReverseRecord witnesses ...");

            let mut prev_root = high_level::load_cell_data(input_cells[0], Source::Input)?;
            let latest_root = high_level::load_cell_data(output_cells[0], Source::Output)?;

            let witness_parser = ReverseRecordWitnessesParser::new()?;
            for witness_ret in witness_parser.iter() {
                if let Err(e) = witness_ret {
                    return Err(e);
                }
                let witness = witness_ret.unwrap();

                verify_sign(&sign_lib, &witness)?;
                smt_verify_reverse_record_proof(&prev_root, &witness)?;

                prev_root = witness.next_root.to_vec();
            }

            das_assert!(
                latest_root == prev_root,
                ErrorCode::SMTNewRootMismatch,
                "outputs[{}] The SMT root in the ReverseRecordRootCell is mismatched.(expected: 0x{}, result: 0x{})",
                output_cells[0],
                util::hex_string(&prev_root),
                util::hex_string(&latest_root)
            )
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}

fn verify_has_some_lock_in_white_list(start_from: usize, white_list: &[[u8; 32]]) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if there is any lock in the inputs exist in the SMT white list.");

    // debug!(
    //     "white_list = {:?}",
    //     white_list.iter().map(|v| util::hex_string(v)).collect::<Vec<String>>()
    // );

    let mut i = start_from;
    loop {
        let result = high_level::load_cell_lock_hash(i, Source::Input);
        match result {
            Ok(input_lock_hash) => {
                debug!(
                    "Verify if the lock hash 0x{} in white list.",
                    util::hex_string(&input_lock_hash)
                );

                if white_list.contains(&input_lock_hash) {
                    return Ok(());
                }
            }
            Err(_) => break,
        }
        i += 1;
    }

    warn!("Can not find any lock in the inputs exist in the SMT white list.");
    Err(code_to_error!(ErrorCode::SMTWhiteListTheLockIsNotFound))
}

fn verify_sign(sign_lib: &SignLib, witness: &ReverseRecordWitness) -> Result<(), Box<dyn ScriptError>> {
    if cfg!(feature = "dev") {
        // CAREFUL Proof verification has been skipped in development mode.
        debug!(
            "  witnesses[{:>2}] Skip verifying the witness.reverse_record.signature is valid.",
            witness.index
        );
        return Ok(());
    }

    debug!(
        "  witnesses[{:>2}] Verify if the witness.reverse_record.signature is valid.",
        witness.index
    );

    let das_lock_type = match witness.sign_type {
        DasLockType::ETH | DasLockType::ETHTypedData | DasLockType::TRON | DasLockType::Doge => witness.sign_type,
        _ => {
            warn!(
                "  witnesses[{:>2}] Parsing das-lock(witness.reverse_record.lock.args) algorithm failed (maybe not supported for now), but it is required in this transaction.",
                witness.index
            );
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
    };

    let nonce = if let Some(prev_nonce) = witness.prev_nonce {
        prev_nonce + 1
    } else {
        1
    };
    let account = witness.next_account.as_bytes().to_vec();
    let signature = witness.signature.as_slice().to_vec();
    let args = witness.address_payload.as_slice().to_vec();
    let data = [nonce.to_le_bytes().to_vec(), account].concat();
    let message = sign_lib.gen_digest(das_lock_type, data).map_err(|_| {
        warn!(
            "  witnesses[{:>2}] The lock type {} is still not supported.",
            witness.index,
            das_lock_type.to_string()
        );
        code_to_error!(ErrorCode::SubAccountSigVerifyError)
    })?;
    let ret = sign_lib.validate_str(das_lock_type, 0i32, message.clone(), message.len(), signature, args);
    match ret {
        Err(_error_code) if _error_code == DasDynamicLibError::UndefinedDasLockType as i32 => {
            warn!(
                "  witnesses[{:>2}] The signature algorithm has not been supported",
                witness.index
            );
            Err(code_to_error!(ErrorCode::HardCodedError))
        }
        Err(_error_code) => {
            warn!(
                "  witnesses[{:>2}] The witness.signature is invalid, the error_code returned by dynamic library is: {}",
                witness.index, _error_code
            );
            Err(code_to_error!(ErrorCode::SubAccountSigVerifyError))
        }
        _ => {
            debug!("  witnesses[{:>2}] The witness.signature is valid.", witness.index);
            Ok(())
        }
    }
}

fn gen_smt_value(nonce: u32, account: &[u8]) -> [u8; 32] {
    let raw = [nonce.to_le_bytes().to_vec(), account.to_vec()].concat();
    util::blake2b_256(raw).into()
}

fn smt_verify_reverse_record_proof(
    prev_root: &[u8],
    witness: &ReverseRecordWitness,
) -> Result<(), Box<dyn ScriptError>> {
    let key = util::blake2b_256(&witness.address_payload);
    let proof = witness.proof.as_slice();

    debug!(
        "  witnesses[{}] Verify if the SMT proof for key 0x{} .",
        witness.index,
        util::hex_string(&key)
    );

    // debug!("  key: 0x{}", util::hex_string(&key));
    // debug!("    proof: 0x{}", util::hex_string(proof));
    // debug!("    prev_root: 0x{}", util::hex_string(prev_root));

    let prev_val: [u8; 32] = if witness.prev_nonce.is_none() {
        [0u8; 32]
    } else {
        gen_smt_value(witness.prev_nonce.unwrap(), witness.prev_account.as_bytes())
    };
    // debug!("    prev_value: 0x{}", util::hex_string(&prev_val));
    verifiers::common::verify_smt_proof(key, prev_val, prev_root.try_into().unwrap(), proof)?;

    let next_val: [u8; 32] = if witness.prev_nonce.is_none() {
        gen_smt_value(1, witness.next_account.as_bytes())
    } else {
        gen_smt_value(witness.prev_nonce.unwrap() + 1, witness.next_account.as_bytes())
    };
    // debug!("    next_root: 0x{}", util::hex_string(&witness.next_root));
    // debug!("    next_value: 0x{}", util::hex_string(&next_val));
    verifiers::common::verify_smt_proof(key, next_val, witness.next_root, proof)?;

    Ok(())
}
