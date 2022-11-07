use serde_json::json;

use crate::util::constants::*;
use crate::util::since_util::SinceFlag;
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
fn yyy() {
    println!("1 day: {:?}", gen_since(SinceFlag::Relative, SinceFlag::Timestamp, DAY_SEC).expect(""));
    // println!("1 day: {:064b}", gen_since(SinceFlag::Relative, SinceFlag::Timestamp, DAY_SEC).expect(""));
    println!("1 hour: {:?}", gen_since(SinceFlag::Relative, SinceFlag::Timestamp, HOUR_SEC).expect(""));
    // println!("1 hour: {:064b}", gen_since(SinceFlag::Relative, SinceFlag::Timestamp, HOUR_SEC).expect(""));
}
