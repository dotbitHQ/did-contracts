use std::env;

use ckb_chain_spec::consensus::TYPE_ID_CODE_HASH;
use ckb_types::core::ScriptHashType;
use ckb_types::packed;
use das_types::prelude::*;
use walkdir::WalkDir;

use super::ckb_types_relay::*;

#[test]
fn gen_type_id_table() {
    let mut hash_list = Vec::new();

    for path in ["../deployed-scripts", "../build/debug"].iter() {
        for entry in WalkDir::new(path)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_dir() {
                continue;
            }

            let filepath = entry.path();
            let filename = filepath.file_name().unwrap().to_str().unwrap().to_owned();

            if filename.starts_with(".") {
                continue;
            }

            let args = {
                // Padding args to 32 bytes, because it is convenient to use 32 bytes as the real args are also 32 bytes.
                let mut buf = [0u8; 32];
                let len = buf.len();
                let bytes = filename.as_bytes();

                if bytes.len() >= len {
                    buf.copy_from_slice(&bytes[..32]);
                } else {
                    let (_, right) = buf.split_at_mut(len - bytes.len());
                    right.copy_from_slice(bytes);
                }
                buf
            };
            let args_bytes = args.iter().map(|v| Byte::new(*v)).collect::<Vec<_>>();
            let type_ = script_build(
                script_new_builder()
                    .code_hash(byte32_new(TYPE_ID_CODE_HASH.as_bytes()))
                    .hash_type(ScriptHashType::Type.into())
                    .args(packed::Bytes::new_builder().set(args_bytes).build()),
            );

            hash_list.push((filename, type_.calc_script_hash()));
        }
    }

    println!("====== Print hash of all scripts ======");
    println!(
        "{:>30}: {}",
        "Currently runs at",
        env::current_dir().unwrap().to_str().unwrap()
    );
    for (filename, hash) in hash_list {
        println!("{:>32}: {}", filename, hash);
    }
}
