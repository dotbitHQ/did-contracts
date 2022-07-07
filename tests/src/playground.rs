use crate::util::{template_common_cell::*, template_generator::*, template_parser::*};

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always-success", false);
    template.push_contract_cell("playground", false);
    // template.push_shared_lib_cell("ckb_smt.so", false);
    template.push_shared_lib_cell("eth_sign.so", false);
    template.push_shared_lib_cell("secp256k1_data", true);
    template
}

#[test]
fn xxx_playground() {
    let mut template = init("playground");

    push_input_playground_cell(&mut template);

    test_tx(template.as_json());
}
