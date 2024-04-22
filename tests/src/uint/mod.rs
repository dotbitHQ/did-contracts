use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn init(name: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("unit_test", Some(name.as_bytes().to_vec()));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    push_input_test_env_cell(&mut template);

    template
}

fn test(name: &str) {
    let template = init(name);
    test_tx(template.as_json());
}

fn perf(name: &str) {
    let template = init(name);
    perf_tx(template.as_json());
}

#[test]
fn test_uint_basic_interface() {
    test("test_uint_basic_interface");
}

#[test]
fn test_uint_safty() {
    test("test_uint_safty");
}

#[test]
fn perf_uint_price_formula() {
    perf("perf_uint_price_formula");
}
