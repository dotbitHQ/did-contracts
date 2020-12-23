// Suppress warning here is because it is mistakenly treat the code as dead code when running unit tests.
#![allow(dead_code)]

use super::*;
use ckb_testtool::context::Context;
use ckb_tool::ckb_hash::blake2b_256;
use ckb_tool::ckb_jsonrpc_types as rpc_types;
use ckb_tool::ckb_types::{
    bytes::Bytes, core::TransactionBuilder, h256, packed::*, prelude::*, H160, H256,
};
use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    pub static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}

pub fn deploy_contract(context: &mut Context, binary_name: &str) -> OutPoint {
    let contract_bin: Bytes = Loader::default().load_binary(binary_name);
    context.deploy_cell(contract_bin)
}

pub fn deploy_builtin_contract(context: &mut Context, binary_name: &str) -> OutPoint {
    let contract_bin: Bytes = Loader::with_deployed_scripts().load_binary(binary_name);
    context.deploy_cell(contract_bin)
}

pub fn mock_script(context: &mut Context, out_point: OutPoint, args: Bytes) -> (Script, CellDep) {
    let script = context
        .build_script(&out_point, args)
        .expect("Build script failed, can not find cell of script.");
    let cell_dep = CellDep::new_builder().out_point(out_point).build();

    (script, cell_dep)
}

pub fn mock_cell(
    context: &mut Context,
    capacity: u64,
    lock_script: Script,
    type_script: Option<Script>,
    bytes: Option<Bytes>,
) -> OutPoint {
    let data;
    if bytes.is_some() {
        data = bytes.unwrap();
    } else {
        data = Bytes::new();
    }

    context.create_cell(
        CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script)
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build(),
        data,
    )
}

pub fn mock_input(out_point: OutPoint, since: Option<u64>) -> CellInput {
    let mut builder = CellInput::new_builder().previous_output(out_point);

    if let Some(data) = since {
        builder = builder.since(data.pack());
    }

    builder.build()
}

pub fn mock_output(capacity: u64, lock_script: Script, type_script: Option<Script>) -> CellOutput {
    CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script)
        .type_(ScriptOpt::new_builder().set(type_script).build())
        .build()
}

pub fn serialize_signature(signature: &secp256k1::recovery::RecoverableSignature) -> [u8; 65] {
    let (recov_id, data) = signature.serialize_compact();
    let mut signature_bytes = [0u8; 65];
    signature_bytes[0..64].copy_from_slice(&data[0..64]);
    signature_bytes[64] = recov_id.to_i32() as u8;
    signature_bytes
}

pub type SignerFn = Box<
    dyn FnMut(&HashSet<H160>, &H256, &rpc_types::Transaction) -> Result<Option<[u8; 65]>, String>,
>;

pub fn get_privkey_signer(privkey: secp256k1::SecretKey) -> SignerFn {
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let lock_arg = H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20])
        .expect("Generate hash(H160) from pubkey failed");
    Box::new(
        move |lock_args: &HashSet<H160>, message: &H256, _tx: &rpc_types::Transaction| {
            if lock_args.contains(&lock_arg) {
                if message == &h256!("0x0") {
                    Ok(Some([0u8; 65]))
                } else {
                    let message = secp256k1::Message::from_slice(message.as_bytes())
                        .expect("Convert to secp256k1 message failed");
                    let signature = SECP256K1.sign_recoverable(&message, &privkey);
                    Ok(Some(serialize_signature(&signature)))
                }
            } else {
                Ok(None)
            }
        },
    )
}
