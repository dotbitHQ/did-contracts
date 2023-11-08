use ckb_types::prelude::Reader;
use serde_json::json;

use crate::util;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always-success", ContractType::Contract);
    template.push_contract_cell("playground", ContractType::Contract);
    // template.push_shared_lib_cell("ckb_smt.so", false);
    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);

    template.push_header_deps(json!({
        "height": HEIGHT,
        "timestamp": TIMESTAMP,
    }));

    template
}

#[test]
fn xxx_playground() {
    let mut template = init("playground");

    push_input_playground_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn xxx_gen_fake_das_lock() {
    println!("Transfer whitelist:");
    println!(
        "{}",
        util::bytes_to_hex(
            gen_fake_das_lock(&util::gen_das_lock_args(DP_TRANSFER_WHITELIST_1, None))
                .as_reader()
                .as_slice()
        )
    );
    println!(
        "{}",
        util::bytes_to_hex(
            gen_fake_das_lock(&util::gen_das_lock_args(DP_TRANSFER_WHITELIST_2, None))
                .as_reader()
                .as_slice()
        )
    );
    println!(
        "{}",
        util::bytes_to_hex(
            gen_fake_das_lock(&util::gen_das_lock_args(DP_TRANSFER_WHITELIST_3, None))
                .as_reader()
                .as_slice()
        )
    );

    println!("Recycle whitelist:");
    println!(
        "{}",
        util::bytes_to_hex(
            gen_fake_das_lock(&util::gen_das_lock_args(DP_RECYCLE_WHITELIST_1, None))
                .as_reader()
                .as_slice()
        )
    );
    println!(
        "{}",
        util::bytes_to_hex(
            gen_fake_das_lock(&util::gen_das_lock_args(DP_RECYCLE_WHITELIST_2, None))
                .as_reader()
                .as_slice()
        )
    );
    println!(
        "{}",
        util::bytes_to_hex(
            gen_fake_das_lock(&util::gen_das_lock_args(DP_RECYCLE_WHITELIST_3, None))
                .as_reader()
                .as_slice()
        )
    );
}
