use super::{constants::*, util};
use ckb_tool::{
    ckb_hash::blake2b_256,
    ckb_types::{bytes, prelude::Pack},
    faster_hex::hex_string,
};
use das_sorted_list::DasSortedList;
use das_types::{constants::*, packed::*, prelude::*, util as das_util};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

fn gen_always_success_lock(lock_args: &str) -> Script {
    Script::new_builder()
        .code_hash(Hash::try_from(ALWAYS_SUCCESS_CODE_HASH.to_vec()).unwrap())
        .hash_type(Byte::new(1))
        .args(Bytes::from(&util::hex_to_bytes(lock_args).unwrap()[..]))
        .build()
}

fn gen_price_config(length: u8, new_price: u64, renew_price: u64) -> PriceConfig {
    PriceConfig::new_builder()
        .length(Uint8::from(length))
        .new(Uint64::from(new_price))
        .renew(Uint64::from(renew_price))
        .build()
}

fn gen_char_set(name: CharSetType, global: u8, chars: Vec<&str>) -> CharSet {
    let mut builder = CharSet::new_builder()
        .name(Uint32::from(name as u32))
        .global(Uint8::from(global));

    let mut chars_builder = Chars::new_builder();
    for char in chars {
        chars_builder = chars_builder.push(Bytes::from(char.as_bytes()));
    }
    builder = builder.chars(chars_builder.build());

    builder.build()
}

fn gen_char_sets() -> CharSetList {
    CharSetList::new_builder()
        .push(gen_char_set(CharSetType::Emoji, 1, vec!["😂", "👍", "✨"]))
        .push(gen_char_set(
            CharSetType::Digit,
            1,
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
        ))
        .push(gen_char_set(
            CharSetType::En,
            0,
            vec![
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F",
                "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V",
                "W", "X", "Y", "Z",
            ],
        ))
        .build()
}

fn gen_type_id_table() -> TypeIdTable {
    let mut builder = TypeIdTable::new_builder();
    for (key, val) in TYPE_ID_TABLE.iter() {
        builder = match *key {
            "apply-register-cell-type" => {
                builder.apply_register_cell(util::hex_to_hash(val).unwrap())
            }
            "pre-account-cell-type" => builder.pre_account_cell(util::hex_to_hash(val).unwrap()),
            "account-cell-type" => builder.account_cell(util::hex_to_hash(val).unwrap()),
            "ref-cell-type" => builder.ref_cell(util::hex_to_hash(val).unwrap()),
            "proposal-cell-type" => builder.proposal_cell(util::hex_to_hash(val).unwrap()),
            _ => builder,
        }
    }

    builder.build()
}

fn gen_account_char(char: &str, char_set_type: CharSetType) -> AccountChar {
    AccountChar::new_builder()
        .char_set_name(Uint32::from(char_set_type as u32))
        .bytes(Bytes::from(char.as_bytes()))
        .build()
}

fn gen_proposal_item(account: &str, item_type: &ProposalSliceItemType, next: &str) -> ProposalItem {
    let account_id = AccountId::try_from(account_to_id_bytes(account)).unwrap();
    let mut builder = ProposalItem::new_builder()
        .account_id(account_id)
        .item_type(Uint8::from(*item_type as u8));

    if !next.is_empty() {
        let next_account_id = AccountId::try_from(account_to_id_bytes(next)).unwrap();
        builder = builder.next(next_account_id);
    }

    builder.build()
}

fn gen_slices(slices: &Vec<Vec<(&str, ProposalSliceItemType, &str)>>) -> SliceList {
    let mut sl_list = SliceList::new_builder();
    for slice in slices {
        if slice.len() <= 1 {
            panic!("Slice must has more than one item.")
        }

        let mut sl = SL::new_builder();
        let mut next_of_first_item = "";
        for (index, (account, item_type, next)) in slice.iter().enumerate() {
            // When it is the first item, saving its next.
            if index == 0 {
                next_of_first_item = next;
                let (next, _, _) = slice.get(index + 1).unwrap();
                sl = sl.push(gen_proposal_item(account, item_type, next));
            // When it is the last item, use next_of_first_item as its next.
            } else if index == slice.len() - 1 {
                sl = sl.push(gen_proposal_item(account, item_type, next_of_first_item));
            // When it is the items between the first and the last, using its next item's account_id as next.
            } else {
                let (next, _, _) = slice.get(index + 1).unwrap();
                sl = sl.push(gen_proposal_item(account, item_type, next));
            }
        }
        sl_list = sl_list.push(sl.build());
    }
    sl_list.build()
}

pub fn gen_account_chars(chars: Vec<&str>) -> AccountChars {
    let mut builder = AccountChars::new_builder();
    for char in chars {
        // Filter empty chars come from str.split("").
        if char.is_empty() {
            continue;
        }

        if char.len() != 1 {
            builder = builder.push(gen_account_char(char, CharSetType::Emoji))
        } else {
            let raw_char = char.chars().next().unwrap();
            if raw_char.is_digit(10) {
                builder = builder.push(gen_account_char(char, CharSetType::Digit))
            } else {
                builder = builder.push(gen_account_char(char, CharSetType::En))
            }
        }
    }

    builder.build()
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
    for account in accounts.iter() {
        let account_id = bytes::Bytes::from(util::account_to_id(account.as_bytes()));
        account_id_map.insert(account_id.clone(), *account);
        account_id_list.push(account_id);
    }

    let sorted_list = DasSortedList::new(account_id_list);
    println!("====== Accounts list ======");
    for item in sorted_list.items() {
        let account = account_id_map.get(item).unwrap();
        println!("{} => {}", account, item.pack());
    }

    let accounts = vec![
        "inviter_01.bit",
        "inviter_02.bit",
        "inviter_03.bit",
        "channel_01.bit",
        "channel_02.bit",
        "channel_03.bit",
        "proposer_01.bit",
        "proposer_02.bit",
        "proposer_03.bit",
    ];
    println!("====== Special accounts list ======");
    for account in accounts.into_iter() {
        let id = bytes::Bytes::from(util::account_to_id(account.as_bytes()));
        println!("{} => {}", account, id.pack());
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

    util::hex_to_bytes(account_id).unwrap().to_vec()
}

fn bytes_to_hex(input: Bytes) -> String {
    "0x".to_string() + &hex_string(input.as_reader().raw_data()).unwrap()
}

pub struct TemplateGenerator {
    pub header_deps: Vec<Value>,
    pub cell_deps: Vec<Value>,
    pub inputs: Vec<Value>,
    pub outputs: Vec<Value>,
    pub witnesses: Vec<String>,
    pub prices: HashMap<u8, PriceConfig>,
}

impl TemplateGenerator {
    pub fn new(action: &str, params_opt: Option<Bytes>) -> TemplateGenerator {
        let witness = das_util::wrap_action_witness(action, params_opt);

        let mut prices = HashMap::new();
        prices.insert(1u8, gen_price_config(1, 12_000_000, 1_200_000));
        prices.insert(2u8, gen_price_config(2, 11_000_000, 1_100_000));
        prices.insert(3u8, gen_price_config(3, 10_000_000, 1_000_000));
        prices.insert(4u8, gen_price_config(4, 9_000_000, 900_000));
        prices.insert(5u8, gen_price_config(5, 8_000_000, 800_000));
        prices.insert(6u8, gen_price_config(6, 7_000_000, 700_000));
        prices.insert(7u8, gen_price_config(7, 6_000_000, 600_000));
        prices.insert(8u8, gen_price_config(8, 5_000_000, 500_000));

        TemplateGenerator {
            header_deps: Vec::new(),
            cell_deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            witnesses: vec![bytes_to_hex(witness)],
            prices,
        }
    }

    pub fn push_witness<T: Entity>(
        &mut self,
        data_type: DataType,
        output_opt: Option<(u32, u32, T)>,
        input_opt: Option<(u32, u32, T)>,
        dep_opt: Option<(u32, u32, T)>,
    ) {
        let witness = das_util::wrap_data_witness(data_type, output_opt, input_opt, dep_opt);
        self.witnesses.push(bytes_to_hex(witness));
    }

    pub fn push_witness_with_group(
        &mut self,
        data_type: DataType,
        group: Source,
        entity: (u32, u32, impl Entity),
    ) {
        let witness = match group {
            Source::Input => das_util::wrap_data_witness(data_type, None, Some(entity), None),
            Source::Output => das_util::wrap_data_witness(data_type, Some(entity), None, None),
            _ => das_util::wrap_data_witness(data_type, None, None, Some(entity)),
        };
        self.witnesses.push(bytes_to_hex(witness));
    }

    pub fn push_cell(
        &mut self,
        capacity: u64,
        lock_script: Value,
        type_script: Value,
        data: Option<Bytes>,
        source: Source,
    ) {
        let mut value;
        if let Some(tmp_data) = data {
            value = json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": bytes_to_hex(tmp_data.clone())
            });
        } else {
            value = json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script
            });
        }

        if source == Source::Input {
            value = json!({
                "previous_output": value,
                "since": "0x"
            });
        }

        match source {
            Source::HeaderDep => self.header_deps.push(value),
            Source::CellDep => self.cell_deps.push(value),
            Source::Input => self.inputs.push(value),
            Source::Output => self.outputs.push(value),
        }
    }

    pub fn push_time_cell(&mut self, index: u8, timestamp: u64, capacity: u64, source: Source) {
        let index = Bytes::from(vec![index]);
        let timestamp = Uint64::from(timestamp);

        let raw = [
            index.as_reader().raw_data(),
            timestamp.as_reader().raw_data(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        let lock_script = json!({
            "code_hash": "{{always_success}}"
        });
        let type_script = json!({
            "code_hash": "0x0100000000000000000000000000000000000000000000000000000000000000",
            "hash_type": "type"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);
    }

    pub fn push_apply_register_cell(
        &mut self,
        pubkey_hash: &str,
        account: &AccountChars,
        timestamp: u64,
        capacity: u64,
        source: Source,
    ) {
        let pubkey_hash = util::hex_to_bytes(pubkey_hash).unwrap();
        let mut account_bytes = account.as_readable();
        account_bytes.append(&mut ".bit".as_bytes().to_vec());
        let hash_of_account = Hash::new_unchecked(
            blake2b_256(
                [pubkey_hash, bytes::Bytes::from(account_bytes)]
                    .concat()
                    .as_slice(),
            )
            .to_vec()
            .into(),
        );

        let raw = [
            hash_of_account.as_reader().raw_data(),
            Uint64::from(timestamp).as_reader().raw_data(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        let lock_script = json!({
            "code_hash": "{{always_success}}"
        });
        let type_script = json!({
            "code_hash": "{{apply-register-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);
    }

    pub fn push_ref_cell(&mut self, lock_args: &str, account: &str, capacity: u64, source: Source) {
        let account_id = Bytes::from(util::account_to_id(account.as_bytes()));
        let lock_script = json!({
          "code_hash": "{{always_success}}",
          "args": lock_args
        });
        let type_script = json!({
          "code_hash": "{{ref-cell-type}}",
          "args": bytes_to_hex(account_id)
        });

        self.push_cell(capacity, lock_script, type_script, None, source);
    }

    pub fn gen_config_cell_data(&mut self) -> (Bytes, ConfigCellData) {
        let mut price_config_list_builder = PriceConfigList::new_builder();
        for (_, price) in self.prices.iter() {
            price_config_list_builder = price_config_list_builder.push(price.to_owned());
        }

        let entity = ConfigCellData::new_builder()
            .reserved_account_filter(Bytes::default())
            .proposal_min_confirm_require(Uint8::from(4))
            .proposal_min_extend_interval(Uint8::from(2))
            .proposal_min_recycle_interval(Uint8::from(6))
            .proposal_max_account_affect(Uint32::from(50))
            .proposal_max_pre_account_contain(Uint32::from(50))
            .apply_min_waiting_time(Uint32::from(60))
            .apply_max_waiting_time(Uint32::from(86400))
            .account_max_length(Uint32::from(1000))
            .price_configs(price_config_list_builder.build())
            .char_sets(gen_char_sets())
            .min_ttl(Uint32::from(300))
            .closing_limit_of_primary_market_auction(Uint32::from(86400))
            .closing_limit_of_secondary_market_auction(Uint32::from(86400))
            .type_id_table(gen_type_id_table())
            .build();

        // Generate the cell structure of ConfigCell.
        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    pub fn push_config_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, impl Entity)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}",
          "args": CONFIG_LOCK_ARGS
        });
        let type_script = json!({
          "code_hash": "{{config-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::ConfigCellData, source, entity);
        }
    }

    pub fn gen_pre_account_cell_data(
        &mut self,
        account_chars: &AccountChars,
        owner_lock_args: &str,
        refund_lock_args: &str,
        inviter_wallet: &str,
        channel_wallet: &str,
        quote: u64,
        created_at: u64,
    ) -> (Bytes, PreAccountCellData) {
        let account_length = if account_chars.len() > 8 {
            8u8
        } else {
            account_chars.len() as u8
        };

        let price = self.prices.get(&account_length).unwrap();
        let entity = PreAccountCellData::new_builder()
            .account(account_chars.to_owned())
            .owner_lock(gen_always_success_lock(owner_lock_args))
            .refund_lock(gen_always_success_lock(refund_lock_args))
            .inviter_wallet(Bytes::from(account_to_id_bytes(inviter_wallet)))
            .channel_wallet(Bytes::from(account_to_id_bytes(channel_wallet)))
            .price(price.to_owned())
            .quote(Uint64::from(quote))
            .created_at(Timestamp::from(created_at))
            .build();

        let mut account = account_chars.as_readable();
        account.append(&mut ".bit".as_bytes().to_vec());
        let id = util::account_to_id(account.as_slice());

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [hash.as_reader().raw_data(), id.as_slice()].concat();
        let cell_data = Bytes::from(raw);

        (cell_data, entity)
    }

    pub fn push_pre_account_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, impl Entity)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}"
        });
        let type_script = json!({
          "code_hash": "{{pre-account-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::PreAccountCellData, source, entity);
        }
    }

    pub fn gen_account_cell_data(
        &mut self,
        account_chars: &AccountChars,
        owner_lock_args: &str,
        manager_lock_args: &str,
        next: bytes::Bytes,
        registered_at: u64,
        expired_at: u64,
    ) -> (Bytes, AccountCellData) {
        let mut account = account_chars.as_readable();
        account.append(&mut ".bit".as_bytes().to_vec());
        let id = util::account_to_id(account.as_slice());

        let entity = AccountCellData::new_builder()
            .id(AccountId::try_from(id.clone()).unwrap())
            .account(account_chars.to_owned())
            .owner_lock(gen_always_success_lock(owner_lock_args))
            .manager_lock(gen_always_success_lock(manager_lock_args))
            .status(Uint8::from(0))
            .build();

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [
            hash.as_reader().raw_data(),
            id.as_slice(),
            &next[..],
            &registered_at.to_le_bytes()[..],
            &expired_at.to_le_bytes()[..],
            account.as_slice(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        (cell_data, entity)
    }

    pub fn push_account_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, AccountCellData)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}"
        });
        let type_script = json!({
          "code_hash": "{{account-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::AccountCellData, source, entity);
        }
    }

    pub fn gen_proposal_cell_data(
        &mut self,
        proposer_lock_args: &str,
        proposer_wallet: &str,
        slices: &Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    ) -> (Bytes, ProposalCellData) {
        let entity = ProposalCellData::new_builder()
            .proposer_lock(gen_always_success_lock(proposer_lock_args))
            .proposer_wallet(Bytes::from(account_to_id_bytes(proposer_wallet)))
            .slices(gen_slices(slices))
            .build();

        let cell_data = Bytes::from(&blake2b_256(entity.as_slice())[..]);

        (cell_data, entity)
    }

    pub fn push_proposal_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, impl Entity)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}"
        });
        let type_script = json!({
          "code_hash": "{{proposal-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::ProposalCellData, source, entity);
        }
    }

    pub fn pretty_print(&self) {
        let data = json!({
            "cell_deps": self.cell_deps,
            "inputs": self.inputs,
            "outputs": self.outputs,
            "witnesses": self.witnesses,
        });

        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    }
}
