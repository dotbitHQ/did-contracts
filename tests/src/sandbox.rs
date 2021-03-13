use super::util::{hex_to_bytes, sandbox::Sandbox};
use ckb_tool::ckb_types::{packed::OutPoint, prelude::Entity};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[test]
fn test_sandbox() {
    let out_point_table = before_each();

    let mut tx_json = String::new();
    File::open(format!("./tx.json"))
        .expect("Expect tx.json exists.")
        .read_to_string(&mut tx_json)
        .expect("Expect tx.json file readable.");

    let mut sandbox = Sandbox::new(out_point_table, "http://127.0.0.1:8214", &tx_json).unwrap();

    let cycles = sandbox.run().unwrap();

    println!("Run transaction costs: {} cycles", cycles);
}

fn before_each() -> HashMap<OutPoint, String> {
    let raw_out_point_table = vec![
        (
            "apply-register-cell-type",
            "0x3e5acfaa77f2a2a4d110e5e53449f6a23e81f2ab2128c3a176784e3d090b83dc00000000",
        ),
        (
            "pre-account-cell-type",
            "0x4072c144d1c5f989f0784a1ce5d0a12184f0050dd633ac5f972a55b5766cb1f200000000",
        ),
        (
            "account-cell-type",
            "0x86610ae8e8185ee5efe1e7dba785984cbfe0d431855803ec3c8f9589b051f98f00000000",
        ),
        (
            "proposal-cell-type",
            "0x7044220f6b9de6473473f0c548dc30a04564792192970786d6cc6d9b8ab742dd00000000",
        ),
        (
            "ref-cell-type",
            "0x1cbacf9b9221600d30793297e41566fcc09fc362fade48d19558535a5bf5b27600000000",
        ),
        (
            "wallet-cell-type",
            "0x88d1d4a5169860afbfba54965595898866133e117da54d300b74c03c7b2d839f00000000",
        ),
        (
            "always-success",
            "0x88462008b19c9ac86fb9fef7150c4f6ef7305d457d6b200c8852852012923bf100000000",
        ),
        (
            "config-cell-type",
            "0xe940acf7dae23bd133d5ab2a1dd82fbea09c21bf11e152246be9a8c8d174da6b00000000",
        ),
        (
            "secp256k1_blake160_sighash_all",
            "0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f01000000",
        ),
        (
            "secp256k1_data",
            "0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f03000000",
        ),
    ];

    let mut out_point_table: HashMap<OutPoint, String> = HashMap::new();
    for (filename, tx_hash) in raw_out_point_table.iter() {
        let out_point = OutPoint::from_slice(hex_to_bytes(tx_hash).unwrap().as_ref()).unwrap();
        out_point_table.insert(out_point, filename.to_string());
    }

    out_point_table
}
