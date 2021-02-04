use ckb_tool::{
    ckb_chain_spec::consensus::TYPE_ID_CODE_HASH,
    ckb_types::core::ScriptHashType,
    ckb_types::{bytes, packed as ckb_packed, prelude::Pack},
};
use das_types::prelude::*;
use std::env;
use walkdir::WalkDir;

#[test]
fn print_hash_of_all_scripts() {
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

            let type_ = ckb_packed::Script::new_builder()
                .code_hash(ckb_packed::Byte32::new_unchecked(bytes::Bytes::from(
                    TYPE_ID_CODE_HASH.as_bytes(),
                )))
                .hash_type(ScriptHashType::Type.into())
                .args(bytes::Bytes::from(filename.clone()).pack())
                .build();

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
        println!("{:>30}: {}", filename, hash);
    }
}
