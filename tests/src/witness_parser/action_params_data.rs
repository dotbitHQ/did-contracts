use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn init(name: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("unit_test", Some(name.as_bytes().to_vec()));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    template
}

#[test]
fn test_witness_parser_action_params_data_init() {
    let mut template = init("test_witness_parser_action_params_data_init");

    push_input_test_env_cell(&mut template);

    test_tx(template.as_json());
}
