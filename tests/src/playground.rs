use crate::util::{template_common_cell::*, template_generator::*, template_parser::*};

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always-success", ContractType::Contract);
    template.push_contract_cell("playground", ContractType::Contract);
    // template.push_shared_lib_cell("ckb_smt.so", false);
    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);
    template
}

#[test]
fn xxx_playground() {
    let mut template = init("playground");

    push_input_playground_cell(&mut template);

    test_tx(template.as_json());
}
