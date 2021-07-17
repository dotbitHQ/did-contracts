use super::{constants::*, util};
use ckb_tool::{ckb_hash::blake2b_256, ckb_types::prelude::Pack, faster_hex::hex_string};
use das_sorted_list::DasSortedList;
use das_types::{constants::*, packed::*, prelude::*, util as das_util};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryFrom, env, fs::OpenOptions, io::Write, str};

fn gen_always_success_lock(lock_args: &str) -> Script {
    Script::new_builder()
        .code_hash(Hash::try_from(ALWAYS_SUCCESS_CODE_HASH.to_vec()).unwrap())
        .hash_type(Byte::new(1))
        .args(Bytes::from(util::hex_to_bytes(lock_args)))
        .build()
}

fn gen_das_lock_args(owner_pubkey_hash: &str, manager_pubkey_hash_opt: Option<&str>) -> String {
    if let Some(manager_pubkey_hash) = manager_pubkey_hash_opt {
        format!(
            "0x00{}00{}",
            owner_pubkey_hash.trim_start_matches("0x"),
            manager_pubkey_hash.trim_start_matches("0x")
        )
    } else {
        format!("0x00{}", owner_pubkey_hash.trim_start_matches("0x"),)
    }
}

fn gen_price_config(length: u8, new_price: u64, renew_price: u64) -> PriceConfig {
    PriceConfig::new_builder()
        .length(Uint8::from(length))
        .new(Uint64::from(new_price))
        .renew(Uint64::from(renew_price))
        .build()
}

fn gen_type_id_table() -> TypeIdTable {
    let mut builder = TypeIdTable::new_builder();
    for (key, val) in TYPE_ID_TABLE.iter() {
        builder = match *key {
            "account-cell-type" => builder.account_cell(util::hex_to_hash(val).unwrap()),
            "apply-register-cell-type" => {
                builder.apply_register_cell(util::hex_to_hash(val).unwrap())
            }
            "bidding-cell-type" => builder.pre_account_cell(util::hex_to_hash(val).unwrap()),
            "income-cell-type" => builder.income_cell(util::hex_to_hash(val).unwrap()),
            "on-sale-cell-type" => builder.on_sale_cell(util::hex_to_hash(val).unwrap()),
            "pre-account-cell-type" => builder.pre_account_cell(util::hex_to_hash(val).unwrap()),
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

pub fn gen_account_record(
    type_: &str,
    key: &str,
    label: &str,
    value: impl AsRef<[u8]>,
    ttl: u32,
) -> Record {
    Record::new_builder()
        .record_type(Bytes::from(type_.as_bytes()))
        .record_key(Bytes::from(key.as_bytes()))
        .record_label(Bytes::from(label.as_bytes()))
        .record_value(Bytes::from(value.as_ref()))
        .record_ttl(Uint32::from(ttl))
        .build()
}

pub fn gen_account_records(records_param: Vec<AccountRecordParam>) -> Records {
    let mut records = Records::new_builder();
    for record_param in records_param.into_iter() {
        records = records.push(gen_account_record(
            record_param.type_,
            record_param.key,
            record_param.label,
            record_param.value,
            300,
        ));
    }
    records.build()
}

pub fn gen_account_chars(chars: Vec<impl AsRef<str>>) -> AccountChars {
    lazy_static! {
        static ref RE_ZH: Regex = Regex::new(r"^[\u4E00-\u9FA5]+$").unwrap();
    }

    let mut builder = AccountChars::new_builder();
    for char in chars {
        let char = char.as_ref();
        // Filter empty chars come from str.split("").
        if char.is_empty() {
            continue;
        }

        // ⚠️ For testing only, the judgement is not accurate, DO NOT support multiple emoji with more than 4 bytes.
        if char.len() != 1 {
            if RE_ZH.is_match(char) {
                builder = builder.push(gen_account_char(char, CharSetType::ZhHans))
            } else {
                builder = builder.push(gen_account_char(char, CharSetType::Emoji))
            }
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
    for account in accounts.into_iter() {
        let account_id = util::account_to_id_bytes(account);
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

fn bytes_to_hex(input: Bytes) -> String {
    "0x".to_string() + &hex_string(input.as_reader().raw_data()).unwrap()
}

#[derive(Debug, Clone)]
pub struct AccountRecordParam {
    pub type_: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IncomeRecordParam {
    pub belong_to: &'static str,
    pub capacity: u64,
}

macro_rules! gen_config_cell_char_set {
    ($fn_name:ident, $is_global:expr, $file_name:expr, $ret_type:expr) => {
        fn $fn_name(&self) -> (Bytes, Vec<u8>) {
            let mut charsets = Vec::new();
            let lines = util::read_lines($file_name)
                .expect(format!("Expect file ./tests/data/{} exist.", $file_name).as_str());
            for line in lines {
                if let Ok(key) = line {
                    charsets.push(key);
                }
            }

            // Join all record keys with 0x00 byte as entity.
            let mut raw = Vec::new();
            raw.push($is_global); // global status
            for key in charsets {
                raw.extend(key.as_bytes());
                raw.extend(&[0u8]);
            }
            raw = util::prepend_molecule_like_length(raw);

            let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());

            (cell_data, raw)
        }
    };
}

pub struct TemplateGenerator {
    pub header_deps: Vec<Value>,
    pub cell_deps: Vec<Value>,
    pub inputs: Vec<Value>,
    pub outputs: Vec<Value>,
    pub witnesses: Vec<String>,
    pub prices: HashMap<u8, PriceConfig>,
    pub preserved_account_groups: HashMap<u32, (Bytes, Vec<u8>)>,
    pub charsets: HashMap<u32, (Bytes, Vec<u8>)>,
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
            preserved_account_groups: HashMap::new(),
            charsets: HashMap::new(),
        }
    }

    pub fn push_witness<A: Entity, B: Entity, C: Entity>(
        &mut self,
        data_type: DataType,
        output_opt: Option<(u32, u32, A)>,
        input_opt: Option<(u32, u32, B)>,
        dep_opt: Option<(u32, u32, C)>,
    ) {
        let witness = das_util::wrap_data_witness(data_type, output_opt, input_opt, dep_opt);
        self.witnesses.push(bytes_to_hex(witness));
    }

    pub fn push_witness_with_group<T: Entity>(
        &mut self,
        data_type: DataType,
        group: Source,
        entity: (u32, u32, T),
    ) {
        let witness = match group {
            Source::Input => {
                das_util::wrap_data_witness::<T, T, T>(data_type, None, Some(entity), None)
            }
            Source::Output => {
                das_util::wrap_data_witness::<T, T, T>(data_type, Some(entity), None, None)
            }
            _ => das_util::wrap_data_witness::<T, T, T>(data_type, None, None, Some(entity)),
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

    pub fn push_contract_cell(&mut self, contract_filename: &str, deployed: bool) {
        let value;
        if deployed {
            value = json!({
                "tmp_type": "deployed_contract",
                "tmp_file_name": contract_filename
            });
        } else {
            value = json!({
                "tmp_type": "contract",
                "tmp_file_name": contract_filename
            });
        }

        self.cell_deps.push(value)
    }

    pub fn push_oracle_cell(&mut self, index: u8, type_: OracleCellType, data: u64) {
        let mut cell_raw_data = Vec::new();
        cell_raw_data.extend(index.to_be_bytes().iter());
        cell_raw_data.extend(&[type_ as u8]);
        cell_raw_data.extend(data.to_be_bytes().iter());
        let cell_data = Bytes::from(cell_raw_data);

        let lock_script = json!({
            "code_hash": "{{always_success}}"
        });
        let type_script = json!({
            "code_hash": "0x0100000000000000000000000000000000000000000000000000000000000000",
            "hash_type": "type",
            "args": format!("0x{}", hex_string(&[type_ as u8]).expect("Expect &[u8] as inputs"))
        });

        self.push_cell(
            40_000_000_000,
            lock_script,
            type_script,
            Some(cell_data),
            Source::CellDep,
        );
    }

    pub fn push_apply_register_cell(
        &mut self,
        lock_args: &str,
        account: &str,
        height: u64,
        timestamp: u64,
        capacity: u64,
        source: Source,
    ) {
        let hash_of_account = Hash::new_unchecked(
            blake2b_256(
                [&util::hex_to_bytes(lock_args), account.as_bytes()]
                    .concat()
                    .as_slice(),
            )
            .to_vec()
            .into(),
        );

        let raw = [
            hash_of_account.as_reader().raw_data(),
            &height.to_le_bytes(),
            &timestamp.to_le_bytes(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        let lock_script = json!({
            "code_hash": "{{always_success}}",
            "args": lock_args
        });
        let type_script = json!({
            "code_hash": "{{apply-register-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);
    }

    fn gen_config_cell_account(&mut self) -> (Bytes, ConfigCellAccount) {
        let entity = ConfigCellAccount::new_builder()
            .max_length(Uint32::from(20))
            .basic_capacity(Uint64::from(ACCOUNT_BASIC_CAPACITY))
            .prepared_fee_capacity(Uint64::from(ACCOUNT_PREPARED_FEE_CAPACITY))
            .expiration_grace_period(Uint32::from(2_592_000))
            .record_min_ttl(Uint32::from(300))
            .record_size_limit(Uint32::from(5000))
            .transfer_account_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .edit_manager_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .edit_records_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .transfer_account_throttle(Uint32::from(86400))
            .edit_manager_throttle(Uint32::from(3600))
            .edit_records_throttle(Uint32::from(600))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_apply(&mut self) -> (Bytes, ConfigCellApply) {
        let entity = ConfigCellApply::new_builder()
            .apply_min_waiting_block_number(Uint32::from(1))
            .apply_max_waiting_block_number(Uint32::from(5760))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_income(&mut self) -> (Bytes, ConfigCellIncome) {
        let entity = ConfigCellIncome::new_builder()
            .basic_capacity(Uint64::from(20_000_000_000))
            .max_records(Uint32::from(50))
            .min_transfer_capacity(Uint64::from(10_000_000_000))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_main(&mut self) -> (Bytes, ConfigCellMain) {
        let entity = ConfigCellMain::new_builder()
            .status(Uint8::from(1))
            .type_id_table(gen_type_id_table())
            .das_lock_out_point_table(DasLockOutPointTable::default())
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_price(&mut self) -> (Bytes, ConfigCellPrice) {
        let discount_config = DiscountConfig::new_builder()
            .invited_discount(Uint32::from(500))
            .build();

        let mut prices = PriceConfigList::new_builder();
        for (_, price) in self.prices.iter() {
            prices = prices.push(price.to_owned());
        }

        let entity = ConfigCellPrice::new_builder()
            .discount(discount_config)
            .prices(prices.build())
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_proposal(&mut self) -> (Bytes, ConfigCellProposal) {
        let entity = ConfigCellProposal::new_builder()
            .proposal_min_confirm_interval(Uint8::from(4))
            .proposal_min_extend_interval(Uint8::from(2))
            .proposal_min_recycle_interval(Uint8::from(6))
            .proposal_max_account_affect(Uint32::from(50))
            .proposal_max_pre_account_contain(Uint32::from(50))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_profit_rate(&mut self) -> (Bytes, ConfigCellProfitRate) {
        let entity = ConfigCellProfitRate::new_builder()
            .channel(Uint32::from(800))
            .inviter(Uint32::from(800))
            .proposal_create(Uint32::from(400))
            .proposal_confirm(Uint32::from(0))
            .income_consolidate(Uint32::from(100))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_release(&mut self) -> (Bytes, ConfigCellRelease) {
        let data = vec![
            (
                2,
                util::gen_timestamp("2021-06-28 00:00:00"),
                util::gen_timestamp("2021-07-10 00:00:00"),
            ),
            (
                0,
                util::gen_timestamp("2021-06-01 00:00:00"),
                util::gen_timestamp("2021-06-01 00:00:00"),
            ),
        ];

        let mut release_rules = ReleaseRules::new_builder();
        for item in data.into_iter() {
            release_rules = release_rules.push(
                ReleaseRule::new_builder()
                    .length(Uint32::from(item.0))
                    .release_start(Timestamp::from(item.1))
                    .release_end(Timestamp::from(item.2))
                    .build(),
            );
        }

        let entity = ConfigCellRelease::new_builder()
            .release_rules(release_rules.build())
            .build();
        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_record_key_namespace(&mut self) -> (Bytes, Vec<u8>) {
        let mut record_key_namespace = Vec::new();
        let lines = util::read_lines("record_key_namespace.txt")
            .expect("Expect file ./tests/data/record_key_namespace.txt exist.");
        for line in lines {
            if let Ok(key) = line {
                record_key_namespace.push(key);
            }
        }
        record_key_namespace.sort();

        // Join all record keys with 0x00 byte as entity.
        let mut raw = Vec::new();
        for key in record_key_namespace {
            raw.extend(key.as_bytes());
            raw.extend(&[0u8]);
        }
        let raw = util::prepend_molecule_like_length(raw);

        let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());

        (cell_data, raw)
    }

    fn gen_config_cell_preserved_account(
        &mut self,
        data_type: DataType,
    ) -> Option<(Bytes, Vec<u8>)> {
        if self.preserved_account_groups.is_empty() {
            // Load and group preserved accounts
            let mut preserved_accounts_groups: Vec<Vec<Vec<u8>>> =
                vec![Vec::new(); PRESERVED_ACCOUNT_CELL_COUNT as usize];
            let lines = util::read_lines("preserved_accounts.txt")
                .expect("Expect file ./data/preserved_accounts.txt exist.");
            for line in lines {
                if let Ok(account) = line {
                    let account_hash = blake2b_256(account.as_bytes())
                        .get(..ACCOUNT_ID_LENGTH)
                        .unwrap()
                        .to_vec();
                    let index = (account_hash[0] % PRESERVED_ACCOUNT_CELL_COUNT) as usize;

                    preserved_accounts_groups[index].push(account_hash);
                }
            }

            // Store grouped preserved accounts into self.preserved_account_groups
            for (_i, mut group) in preserved_accounts_groups.into_iter().enumerate() {
                // println!("Preserved account group[{}] count: {}", _i, group.len());
                group.sort();
                let mut raw = group.into_iter().flatten().collect::<Vec<u8>>();
                raw = util::prepend_molecule_like_length(raw);

                let data_type = das_util::preserved_accounts_group_to_data_type(_i);
                let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());
                self.preserved_account_groups
                    .insert(data_type as u32, (cell_data, raw));
            }
        }

        self.preserved_account_groups
            .get(&(data_type as u32))
            .map(|item| item.to_owned())
    }

    gen_config_cell_char_set!(
        gen_config_cell_char_set_emoji,
        1,
        "char_set_emoji.txt",
        DataType::ConfigCellCharSetEmoji
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_digit,
        1,
        "char_set_digit.txt",
        DataType::ConfigCellCharSetDigit
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_en,
        1,
        "char_set_en.txt",
        DataType::ConfigCellCharSetEn
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_zh_hans,
        1,
        "char_set_zh_hans.txt",
        DataType::ConfigCellCharSetZhHans
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_zh_hant,
        1,
        "char_set_zh_hant.txt",
        DataType::ConfigCellCharSetZhHant
    );

    pub fn push_config_cell(
        &mut self,
        config_type: DataType,
        push_witness: bool,
        capacity: u64,
        source: Source,
    ) {
        macro_rules! gen_config_data_and_entity_witness {
            ( $method:ident, $config_type:expr ) => {{
                let (cell_data, entity) = self.$method();
                (
                    cell_data,
                    das_util::wrap_entity_witness($config_type, entity),
                )
            }};
        }

        macro_rules! gen_config_data_and_raw_witness {
            ( $gen_fn:ident, $config_type:expr ) => {{
                let (cell_data, raw) = self.$gen_fn();
                (cell_data, das_util::wrap_raw_witness($config_type, raw))
            }};
        }

        let (cell_data, witness) = match config_type {
            DataType::ConfigCellApply => {
                gen_config_data_and_entity_witness!(
                    gen_config_cell_apply,
                    DataType::ConfigCellApply
                )
            }
            DataType::ConfigCellIncome => {
                gen_config_data_and_entity_witness!(
                    gen_config_cell_income,
                    DataType::ConfigCellIncome
                )
            }
            DataType::ConfigCellMain => {
                gen_config_data_and_entity_witness!(gen_config_cell_main, DataType::ConfigCellMain)
            }
            DataType::ConfigCellAccount => {
                gen_config_data_and_entity_witness!(
                    gen_config_cell_account,
                    DataType::ConfigCellAccount
                )
            }
            DataType::ConfigCellPrice => {
                gen_config_data_and_entity_witness!(
                    gen_config_cell_price,
                    DataType::ConfigCellPrice
                )
            }
            DataType::ConfigCellProposal => {
                gen_config_data_and_entity_witness!(
                    gen_config_cell_proposal,
                    DataType::ConfigCellProposal
                )
            }
            DataType::ConfigCellProfitRate => gen_config_data_and_entity_witness!(
                gen_config_cell_profit_rate,
                DataType::ConfigCellProfitRate
            ),
            DataType::ConfigCellRelease => gen_config_data_and_entity_witness!(
                gen_config_cell_release,
                DataType::ConfigCellRelease
            ),
            DataType::ConfigCellRecordKeyNamespace => {
                let (cell_data, raw) = self.gen_config_cell_record_key_namespace();
                (
                    cell_data,
                    das_util::wrap_raw_witness(DataType::ConfigCellRecordKeyNamespace, raw),
                )
            }
            DataType::ConfigCellPreservedAccount00
            | DataType::ConfigCellPreservedAccount01
            | DataType::ConfigCellPreservedAccount02
            | DataType::ConfigCellPreservedAccount03
            | DataType::ConfigCellPreservedAccount04
            | DataType::ConfigCellPreservedAccount05
            | DataType::ConfigCellPreservedAccount06
            | DataType::ConfigCellPreservedAccount07
            | DataType::ConfigCellPreservedAccount08
            | DataType::ConfigCellPreservedAccount09
            | DataType::ConfigCellPreservedAccount10
            | DataType::ConfigCellPreservedAccount11
            | DataType::ConfigCellPreservedAccount12
            | DataType::ConfigCellPreservedAccount13
            | DataType::ConfigCellPreservedAccount14
            | DataType::ConfigCellPreservedAccount15
            | DataType::ConfigCellPreservedAccount16
            | DataType::ConfigCellPreservedAccount17
            | DataType::ConfigCellPreservedAccount18
            | DataType::ConfigCellPreservedAccount19 => {
                match self.gen_config_cell_preserved_account(config_type) {
                    Some((cell_data, raw)) => {
                        (cell_data, das_util::wrap_raw_witness(config_type, raw))
                    }
                    None => panic!("Load preserved accounts from file failed ..."),
                }
            }
            DataType::ConfigCellCharSetEmoji => gen_config_data_and_raw_witness!(
                gen_config_cell_char_set_emoji,
                DataType::ConfigCellCharSetEmoji
            ),
            DataType::ConfigCellCharSetDigit => gen_config_data_and_raw_witness!(
                gen_config_cell_char_set_digit,
                DataType::ConfigCellCharSetDigit
            ),
            DataType::ConfigCellCharSetEn => gen_config_data_and_raw_witness!(
                gen_config_cell_char_set_en,
                DataType::ConfigCellCharSetEn
            ),
            DataType::ConfigCellCharSetZhHans => gen_config_data_and_raw_witness!(
                gen_config_cell_char_set_zh_hans,
                DataType::ConfigCellCharSetZhHans
            ),
            DataType::ConfigCellCharSetZhHant => gen_config_data_and_raw_witness!(
                gen_config_cell_char_set_zh_hant,
                DataType::ConfigCellCharSetZhHant
            ),
            _ => {
                panic!("Not config cell data type.")
            }
        };

        // Create config cell.
        let config_id_hex = hex_string(&(config_type as u32).to_le_bytes()).unwrap();
        let lock_script = json!({
          "code_hash": "{{always_success}}",
          "args": CONFIG_LOCK_ARGS
        });
        let type_script = json!({
          "code_hash": "{{config-cell-type}}",
          "args": format!("0x{}", config_id_hex),
        });
        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if push_witness {
            // Create config cell witness.
            self.witnesses.push(bytes_to_hex(witness));
        }
    }

    pub fn push_config_cell_derived_by_account(
        &mut self,
        account_without_suffix: &str,
        push_witness: bool,
        capacity: u64,
        source: Source,
    ) {
        let first_byte_of_account_hash = blake2b_256(account_without_suffix.as_bytes())[0];
        let index = (first_byte_of_account_hash % PRESERVED_ACCOUNT_CELL_COUNT) as usize;
        let config_type = das_util::preserved_accounts_group_to_data_type(index);

        println!(
            "The first byte of account hash is {:?}, so {:?} will be chosen.",
            first_byte_of_account_hash, config_type
        );

        let (cell_data, witness) = match self.gen_config_cell_preserved_account(config_type) {
            Some((cell_data, raw)) => (cell_data, das_util::wrap_raw_witness(config_type, raw)),
            None => panic!("Can not find preserved account group from the account ..."),
        };

        // Create config cell.
        let config_id_hex = hex_string(&(config_type as u32).to_le_bytes()).unwrap();
        let lock_script = json!({
          "code_hash": "{{always_success}}",
          "args": CONFIG_LOCK_ARGS
        });
        let type_script = json!({
          "code_hash": "{{config-cell-type}}",
          "args": format!("0x{}", config_id_hex),
        });
        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if push_witness {
            // Create config cell witness.
            self.witnesses.push(bytes_to_hex(witness));
        }
    }

    pub fn gen_pre_account_cell_data(
        &mut self,
        account: &str,
        refund_lock_args: &str,
        owner_lock_args: &str,
        inviter_lock_args: &str,
        channel_lock_args: &str,
        quote: u64,
        invited_discount: u32,
        created_at: u64,
    ) -> (Bytes, PreAccountCellData) {
        let account_chars_raw = account[..account.len() - 4]
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        let account_chars = gen_account_chars(account_chars_raw);
        let account_length = if account_chars.len() > 8 {
            8u8
        } else {
            account_chars.len() as u8
        };

        let price = self.prices.get(&account_length).unwrap();
        let mut tmp = util::hex_to_bytes(&gen_das_lock_args(owner_lock_args, None));
        tmp.append(&mut tmp.clone());
        let owner_lock_args = Bytes::from(tmp);
        let entity = PreAccountCellData::new_builder()
            .account(account_chars.to_owned())
            .owner_lock_args(owner_lock_args)
            .refund_lock(gen_always_success_lock(refund_lock_args))
            .inviter_id(Bytes::from(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]))
            .inviter_lock(ScriptOpt::from(gen_always_success_lock(inviter_lock_args)))
            .channel_lock(ScriptOpt::from(gen_always_success_lock(channel_lock_args)))
            .price(price.to_owned())
            .quote(Uint64::from(quote))
            .invited_discount(Uint32::from(invited_discount))
            .created_at(Timestamp::from(created_at))
            .build();

        let id = util::account_to_id(account);

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

    pub fn gen_account_cell_data_v1(
        &mut self,
        account: &str,
        next_account: &str,
        registered_at: u64,
        expired_at: u64,
        records_opt: Option<Records>,
    ) -> (Bytes, AccountCellDataV1) {
        let account_chars_raw = account
            .chars()
            .take(account.len() - 4)
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        let account_chars = gen_account_chars(account_chars_raw);
        let id = util::account_to_id(account);

        let records = match records_opt {
            Some(records) => records,
            None => Records::default(),
        };

        let entity = AccountCellDataV1::new_builder()
            .id(AccountId::try_from(id.clone()).unwrap())
            .account(account_chars.to_owned())
            .registered_at(Uint64::from(registered_at))
            .status(Uint8::from(0))
            .records(records)
            .build();

        let next = util::account_to_id(next_account);

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [
            hash.as_reader().raw_data(),
            id.as_slice(),
            next.as_slice(),
            &expired_at.to_le_bytes()[..],
            account.as_bytes(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        (cell_data, entity)
    }

    pub fn gen_account_cell_data(
        &mut self,
        account: &str,
        next_account: &str,
        registered_at: u64,
        expired_at: u64,
        last_transfer_account_at: u64,
        last_edit_manager_at: u64,
        last_edit_records_at: u64,
        records_opt: Option<Records>,
    ) -> (Bytes, AccountCellData) {
        let account_chars_raw = account
            .chars()
            .take(account.len() - 4)
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        let account_chars = gen_account_chars(account_chars_raw);
        let id = util::account_to_id(account);

        let records = match records_opt {
            Some(records) => records,
            None => Records::default(),
        };

        let entity = AccountCellData::new_builder()
            .id(AccountId::try_from(id.clone()).unwrap())
            .account(account_chars.to_owned())
            .registered_at(Uint64::from(registered_at))
            .last_transfer_account_at(Timestamp::from(last_transfer_account_at))
            .last_edit_manager_at(Timestamp::from(last_edit_manager_at))
            .last_edit_records_at(Timestamp::from(last_edit_records_at))
            .status(Uint8::from(0))
            .records(records)
            .build();

        let next = util::account_to_id(next_account);

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [
            hash.as_reader().raw_data(),
            id.as_slice(),
            next.as_slice(),
            &expired_at.to_le_bytes()[..],
            account.as_bytes(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        (cell_data, entity)
    }

    pub fn gen_root_account_cell_data(&mut self) -> (Bytes, AccountCellData) {
        let id: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let next: Vec<u8> = vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255,
        ];
        let account = AccountChars::default();
        let expired_at = u64::MAX.to_le_bytes();

        let entity = AccountCellData::new_builder()
            .id(AccountId::try_from(id.clone()).unwrap())
            .account(account)
            .registered_at(Uint64::from(0))
            .status(Uint8::from(0))
            .build();

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [
            hash.as_reader().raw_data(),
            id.as_slice(),
            next.as_slice(),
            &expired_at[..],
            &[0],
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        (cell_data, entity)
    }

    pub fn push_account_cell<T: Entity>(
        &mut self,
        owner_lock_args: &str,
        manager_lock_args: &str,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, T)>,
        capacity: u64,
        source: Source,
    ) {
        let args = gen_das_lock_args(owner_lock_args, Some(manager_lock_args));

        let lock_script = json!({
          "code_hash": "{{always_success}}",
          "args": args
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
        created_at_height: u64,
        slices: &Vec<Vec<(&str, ProposalSliceItemType, &str)>>,
    ) -> (Bytes, ProposalCellData) {
        let entity = ProposalCellData::new_builder()
            .proposer_lock(gen_always_success_lock(proposer_lock_args))
            .created_at_height(Uint64::from(created_at_height))
            .slices(gen_slices(slices))
            .build();
        // println!("entity = {:?}", entity);

        let hash = blake2b_256(entity.as_slice());
        // println!("hash = {:?}", hash);
        let cell_data = Bytes::from(&hash[..]);

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

    pub fn gen_income_cell_data(
        &mut self,
        creator: &str,
        records_param: Vec<IncomeRecordParam>,
    ) -> (Bytes, IncomeCellData) {
        let creator = gen_always_success_lock(creator);

        let mut records = IncomeRecords::new_builder();
        for record_param in records_param.into_iter() {
            records = records.push(
                IncomeRecord::new_builder()
                    .belong_to(gen_always_success_lock(record_param.belong_to))
                    .capacity(Uint64::from(record_param.capacity))
                    .build(),
            );
        }

        let entity = IncomeCellData::new_builder()
            .creator(creator)
            .records(records.build())
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    pub fn push_income_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, IncomeCellData)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}"
        });
        let type_script = json!({
          "code_hash": "{{income-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::IncomeCellData, source, entity);
        }
    }

    pub fn push_signall_cell(&mut self, lock_args: &str, capacity: u64, source: Source) {
        let lock_script = json!({
          "code_hash": "{{always_success}}",
          "args": lock_args
        });

        self.push_cell(capacity, lock_script, json!(null), None, source);
    }

    pub fn gen_header() {}

    pub fn as_json(&self) -> serde_json::Value {
        json!({
            "cell_deps": self.cell_deps,
            "inputs": self.inputs,
            "outputs": self.outputs,
            "witnesses": self.witnesses,
        })
    }

    pub fn write_template(&self, filename: &str) {
        let mut filepath = env::current_dir().unwrap();
        filepath.push("templates");
        filepath.push(filename);

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(filepath.clone())
            .expect(format!("Expect file path {:?} to be writable.", filepath).as_str());

        let data = serde_json::to_string_pretty(&self.as_json()).unwrap();
        file.write(data.as_bytes()).expect("Write file failed.");
    }
}
