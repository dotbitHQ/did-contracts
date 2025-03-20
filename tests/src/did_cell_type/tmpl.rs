use std::collections::HashMap;

use das_types::constants::*;
use das_types::packed::*;
use das_types::prelude::*;
use serde_json::{json, Value};

use crate::did_cell_type::utils::*;
use crate::util::constants::*;
use crate::util::smt::SMTWithHistory;
use crate::util::template_generator::*;

pub fn init() -> TemplateGenerator {
    let mut template = TemplateGenerator {
        loaded_contracts: vec![],
        header_deps: Vec::new(),
        cell_deps: Vec::new(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        inner_witnesses: Vec::new(),
        outer_witnesses: Vec::new(),
        sub_account_outer_witnesses: Vec::new(),
        reverse_record_outer_witnesses: Vec::new(),
        sub_account_price_rules_bytes: Vec::new(),
        sub_account_preserved_rules_bytes: Vec::new(),
        prices: HashMap::new(),
        preserved_account_groups: HashMap::new(),
        charsets: HashMap::new(),
        smt_with_history: SMTWithHistory::new(),
        new_sub_account_smt: SMTWithHistory::new(),
    };
    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("did-cell-type", ContractType::Contract);

    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);

    template
}

pub fn push_did_cell_with_args(
    template: &mut TemplateGenerator,
    account: &[u8],
    expire_at: u64,
    num_records: usize,
    source: Source,
    args: [u8; 32],
) {
    let (cell, witness, _, _) = gen_did_cell_with_args(account, expire_at, num_records, args);

    if source == Source::Input {
        template.push_empty_witness();
    }
    if source == Source::Output {
        template.outer_witnesses.push(witness);
    }
    template.push_cell_json(cell, source, None);
}

pub fn gen_did_cell_with_args(
    account: &[u8],
    expire_at: u64,
    num_records: usize,
    args: [u8; 32],
) -> (Value, String, SporeData, DidEntity) {
    let (ent, hash) = gen_did_entity(num_records);
    let dcd = gen_did_cell_data(account, expire_at, &hash);
    let witness = [b"DID".to_vec(), ent.as_bytes().to_vec()].concat();

    let cell = json!({
        "tmp_header": null,
        "tmp_type": "full",
        "capacity": 2160000,
        "lock": {
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        },
        "type": {
            "code_hash": "{{did-cell-type}}",
            "hash_type": "type",
            "args": format!("0x{}", hex::encode(&args))
        },
        "tmp_data": format!("{dcd:#x}"),
    });
    let witness = format!("0x{}", hex::encode(&witness));

    (cell, witness, dcd, ent)
}

pub fn push_did_cell(
    template: &mut TemplateGenerator,
    account: &[u8],
    expire_at: u64,
    num_records: usize,
    source: Source,
) {
    let (cell, witness, _, _) = gen_did_cell(account, expire_at, num_records);

    match source {
        Source::Input | Source::Output => {
            template.outer_witnesses.push(witness);
        }
        Source::CellDep => todo!(),
    }
    template.push_cell_json(cell, source, None);
}

pub fn gen_did_cell(account: &[u8], expire_at: u64, num_records: usize) -> (Value, String, SporeData, DidEntity) {
    let (ent, hash) = gen_did_entity(num_records);
    let dcd = gen_did_cell_data(account, expire_at, &hash);
    let witness = [b"DID".to_vec(), ent.as_bytes().to_vec()].concat();

    let cell = json!({
        "tmp_header": null,
        "tmp_type": "full",
        "capacity": 2160000,
        "lock": {
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        },
        "type": {
            "code_hash": "{{did-cell-type}}",
            "hash_type": "type",
            "args": ""
        },
        "tmp_data": format!("{dcd:#x}"),
    });
    let witness = format!("0x{}", hex::encode(&witness));

    (cell, witness, dcd, ent)
}

pub fn gen_did_cell_with_lock(account: &[u8], lock: Value) -> (Value, String, SporeData, DidEntity) {
    let (ent, hash) = gen_did_entity(2);
    let dcd = gen_did_cell_data(account, u64::MAX, &hash);
    let witness = [b"DID".to_vec(), ent.as_bytes().to_vec()].concat();

    let cell = json!({
        "tmp_header": null,
        "tmp_type": "full",
        "capacity": 2160000,
        "lock": lock,
        "type": {
            "code_hash": "{{did-cell-type}}",
            "hash_type": "type",
            "args": ""
        },
        "tmp_data": format!("{dcd:#x}"),
    });
    let witness = format!("0x{}", hex::encode(&witness));

    (cell, witness, dcd, ent)
}

pub fn push_account_cell(template: &mut TemplateGenerator, account: &[u8], expire_at: u64, source: Source) {
    let hash = [0u8; 32];
    let to_ids = [0u8; 20 * 2];
    let expire_at = expire_at.to_le_bytes();
    let data = [&hash[..], &to_ids[..], &expire_at[..], account].concat();

    let cell = json!({
        "tmp_header": null,
        "tmp_type": "full",
        "capacity": 216000,
        "lock": {
            "code_hash": "{{always_success}}",
            "hash_type": "type",
            "args": ""
        },
        "type": {
            "code_hash": "{{account-cell-type}}",
            "hash_type": "type",
            "args": ""
        },
        "tmp_data": format!("0x{}", hex::encode(data))
    });
    if source == Source::Input {
        template.push_empty_witness();
    }
    template.push_cell_json(cell, source, None);
}
