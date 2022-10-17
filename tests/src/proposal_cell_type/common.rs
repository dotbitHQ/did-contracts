use std::collections::HashMap;

use ckb_types::prelude::Pack;
use das_sorted_list::DasSortedList;
use das_types_std::constants::*;
use serde_json::{json, Value};

use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::{self};

pub struct LockScripts {
    pub inviter_1: Value,
    pub inviter_2: Value,
    pub channel_1: Value,
    pub channel_2: Value,
    pub proposer: Value,
    pub das_wallet: Value,
}

pub fn gen_lock_scripts() -> LockScripts {
    LockScripts {
        inviter_1: json!({
            "code_hash": "{{fake-das-lock}}",
            "args": "0x1111000000000000000000000000000000000000"
        }),
        inviter_2: json!({
            "code_hash": "{{fake-das-lock}}",
            "args": "0x1122000000000000000000000000000000000000"
        }),
        channel_1: json!({
            "code_hash": "{{fake-das-lock}}",
            "args": "0x2211000000000000000000000000000000000000"
        }),
        channel_2: json!({
            "code_hash": "{{fake-das-lock}}",
            "args": "0x2222000000000000000000000000000000000000"
        }),
        proposer: json!({
            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
            "args": COMMON_PROPOSER
        }),
        das_wallet: json!({
            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
            "args": DAS_WALLET_LOCK_ARGS
        }),
    }
}

pub fn gen_account_list() {
    let accounts = vec![
        "das00000.bit",
        "das00001.bit",
        "das00002.bit",
        "das00003.bit",
        "das00004.bit",
        "das00005.bit",
        "das00006.bit",
        "das00007.bit",
        "das00008.bit",
        "das00009.bit",
        "das00010.bit",
        "das00011.bit",
        "das00012.bit",
        "das00013.bit",
        "das00014.bit",
        "das00015.bit",
        "das00016.bit",
        "das00018.bit",
        "das00019.bit",
    ];

    let mut account_id_map = HashMap::new();
    let mut account_id_list = Vec::new();
    for account in accounts.into_iter() {
        let account_id = util::account_to_id(account);
        account_id_map.insert(account_id.clone(), account);
        account_id_list.push(account_id);
    }

    let sorted_list = DasSortedList::new(account_id_list);
    println!("====== Accounts list ======");
    for item in sorted_list.items() {
        let account = account_id_map.get(item).unwrap();
        println!("{} => {}", account, item.pack());
    }
}

pub fn account_to_id_bytes(account: &str) -> Vec<u8> {
    // Sorted testing accounts, generated from gen_account_list().

    let account_id = match account {
        "das00012.bit" => "0x05d2771e6c0366846677ce2e97fe7a78a20ad1f8",
        "das00005.bit" => "0x0eb54a48689ce16b1fe8eaf126f81e9eff558a73",
        "das00009.bit" => "0x100871e1ff8a4cbde1d4673914707a32083e4ce0",
        "das00002.bit" => "0x1710cbaf08cf1fa2dcad206a909c76705970a2ee",
        "das00013.bit" => "0x2ba50252fba902dc5287c8617a98d2b8e0c201d9",
        "das00010.bit" => "0x5998f1666df91f989212994540a51561e1d3dc44",
        "das00004.bit" => "0x5cbc30a5bfd00de7f7c512473f2ff097e7bba50b",
        "das00018.bit" => "0x60e70978fd6456883990ab9a6a0785efdf0d5250",
        "das00008.bit" => "0x6729295b9a936d6c67fd8b84990c9639b01985bd",
        "das00011.bit" => "0x70aa5e4d41c3d557ca847bd10f1efe9b2ca07aca",
        "das00006.bit" => "0x76b1e100d9ff3dc62e75e810be3253bf61d8e794",
        "das00019.bit" => "0x9b992223d5ccd0fca8e17e122e97cff52afaf3ec",
        "das00001.bit" => "0xa2e06438859db0449574d1443ade636a7e6bd09f",
        "das00014.bit" => "0xb209ac25bd48f00b9ae6a1cb7ecff4b58d6c1d07",
        "das00003.bit" => "0xbfab64fccdb83b3316cf3b8faaf6bb5cedef7e4c",
        "das00007.bit" => "0xc9804583fc51c64512c0153264a707c254ae81ff",
        "das00000.bit" => "0xcc4e1b0c31b5f537ad0a91f37c7aea0f47b450f5",
        "das00016.bit" => "0xeb12a2e3eabe2a20015c8bee1b3cfb8577f6360f",
        "das00015.bit" => "0xed912d8f62ce9815d415370c66b00cefc80fcce6",
        // ======
        "das.bit" => "0xb7526803f67ebe70aba631ae3e9560e0cd969c2d",
        "inviter_01.bit" => "0x12dedc00d4e73e7c7c501e386a5514e8a3e94129",
        "inviter_02.bit" => "0xa3281c84335fc37e83315f39ffbb582ba184e433",
        "inviter_03.bit" => "0x75614a21ee56531bf708326d7ae9f053703464d7",
        "channel_01.bit" => "0x0de06eb8bcaa6b33d9ebea5dcdde9f38e98677dc",
        "channel_02.bit" => "0xd6d474d85a9edc9ee26a925b5812c358148e7f78",
        "channel_03.bit" => "0x3781b1590a0ef3b0bbce7513d018e4f636d2b219",
        "proposer_01.bit" => "0xdb352d1d5e245fa09697221b5f7e1bde025f2ee8",
        "proposer_02.bit" => "0xc0d1d7b1b88c584c4c2309d550cfab60cf3b3c7d",
        "proposer_03.bit" => "0x26bcd993ec69922481d26f65d0d778592530de5e",
        // ======
        _ => panic!("Can not find ID of account."),
    };

    // let tmp = [];
    // tmp.iter().for_each(|item| {
    //     println!(
    //         "\"{}\" => \"0x{}\",",
    //         item,
    //         hex_string(blake2b_256(item.as_bytes()).get(..20).unwrap()).unwrap()
    //     )
    // });

    util::hex_to_bytes(account_id)
}

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("proposal-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_oracle_cell(1, OracleCellType::Height, HEIGHT);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProposal, Source::CellDep);

    template
}

pub fn init_with_confirm() -> TemplateGenerator {
    let mut template = init("confirm_proposal");

    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("pre-account-cell-type", ContractType::Contract);
    template.push_contract_cell("income-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);

    template
}

pub fn push_dep_proposal_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": "20_000_000_000",
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{proposal-cell-type}}"
        },
        "witness": {
            "proposer_lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": COMMON_PROPOSER
            },
            "created_at_height": HEIGHT,
            "slices": Value::Null
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_dep(cell, None);
}

pub fn push_input_proposal_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": "20_000_000_000",
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{proposal-cell-type}}"
        },
        "witness": {
            "proposer_lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": COMMON_PROPOSER
            },
            "created_at_height": HEIGHT,
            "slices": Value::Null
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None);
    template.push_empty_witness();
}

pub fn push_output_proposal_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": "20_000_000_000",
        "lock": {
            "code_hash": "{{always_success}}"
        },
        "type": {
            "code_hash": "{{proposal-cell-type}}"
        },
        "witness": {
            "proposer_lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": COMMON_PROPOSER
            },
            "created_at_height": HEIGHT,
            "slices": Value::Null
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, None);
}
