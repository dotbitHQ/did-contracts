use std::convert::TryFrom;

use ckb_hash::blake2b_256;
use das_types::constants::*;
use das_types::packed::*;
use das_types::prelude::*;
use das_types::util::EntityWrapper;
use serde_json::{json, Value};

use crate::util::accounts::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn init(name: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("unit_test", Some(name.as_bytes().to_vec()));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

fn gen_account_cell(outputs_data_opt: Option<String>) -> (Value, EntityWrapper) {
    let entity = AccountCellData::new_builder()
        .id(AccountId::try_from(vec![0; 20]).unwrap())
        .build();

    let lock = parse_json_script_das_lock(
        "",
        &json!({
            "owner_lock_args": OWNER,
            "manager_lock_args": MANAGER
        }),
    );
    // The outputs_data of AccountCell is simplified for witnesses tests, so it IS invalid in other tests.
    let outputs_data = if let Some(val) = outputs_data_opt {
        val
    } else {
        util::bytes_to_hex(&blake2b_256(entity.as_slice()))
    };
    let cell = json!({
        "tmp_type": "full",
        "capacity": util::gen_account_cell_capacity(5),
        "lock": lock,
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "tmp_data": outputs_data
    });

    (cell, EntityWrapper::AccountCellData(entity))
}

#[test]
fn parse_witness_cells() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(None);
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::AccountCellData, 3, Some(entity), None);

    push_input_test_env_cell(&mut template);
    test_tx(template.as_json());
}

#[test]
fn parse_witness_end_with_unknown_witnesses() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(None);
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::AccountCellData, 3, Some(entity), None);
    template
        .outer_witnesses
        .push(String::from("0x11112222333344445555666677778888"));
    template
        .outer_witnesses
        .push(String::from("0x11112222333344445555666677778888"));

    push_input_test_env_cell(&mut template);
    test_tx(template.as_json());
}

#[test]
fn parse_witness_error_cells_data_type() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(None);
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::IncomeCellData, 3, Some(entity), None);

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), ErrorCode::WitnessDataHashOrTypeMissMatch);
}

#[test]
fn parse_witness_error_cells_hash() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(Some(String::from(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )));
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::IncomeCellData, 3, Some(entity), None);

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), ErrorCode::WitnessDataHashOrTypeMissMatch);
}
