use das_types::constants::*;
use das_types::packed::*;
use serde_json::{json, Value};

use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::{self};

pub const BASIC_CAPACITY: u64 = 161;

pub fn init(args: Value) -> TemplateGenerator {
    let action = args["action"].as_str().unwrap_or("transfer_dp");
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);

    template.push_contract_cell("eip712-lib", ContractType::Contract);
    template.push_contract_cell("dpoint-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    // TODO Implement the gen_config_cell_dpoint function in util/template_generator.rs
    // template.push_config_cell(DataType::ConfigCellDPoint, Source::CellDep);

    if action == "transfer_dp" {
        // TODO Add required deps if needed
    }

    template
}
