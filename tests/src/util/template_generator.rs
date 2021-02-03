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

fn gen_price_config(length: u8, new_price: u64, renew_price: u64) -> PriceConfig {
    PriceConfig::new_builder()
        .length(Uint8::from(length))
        .new(Uint64::from(new_price))
        .renew(Uint64::from(renew_price))
        .build()
}

fn gen_price_config_list() -> PriceConfigList {
    // Price unit: USD, accurate to 6 decimal places
    PriceConfigList::new_builder()
        .push(gen_price_config(1, 12_000_000, 1_200_000))
        .push(gen_price_config(2, 11_000_000, 1_100_000))
        .push(gen_price_config(3, 10_000_000, 1_000_000))
        .push(gen_price_config(4, 9_000_000, 900_000))
        .push(gen_price_config(5, 8_000_000, 800_000))
        .push(gen_price_config(6, 7_000_000, 700_000))
        .push(gen_price_config(7, 6_000_000, 600_000))
        .push(gen_price_config(8, 5_000_000, 500_000))
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
            CharSetType::En,
            0,
            vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
                "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
                "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
                "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
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

pub fn gen_account_chars(chars: Vec<&str>) -> AccountChars {
    let mut builder = AccountChars::new_builder();
    for char in chars {
        if char.len() != 1 {
            builder = builder.push(gen_account_char(char, CharSetType::Emoji))
        } else {
            builder = builder.push(gen_account_char(char, CharSetType::En))
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
    ];

    let mut account_id_map = HashMap::new();
    let mut account_id_list = Vec::new();
    for account in accounts.iter() {
        let account_id = bytes::Bytes::from(util::account_to_id(account.as_bytes().to_vec()));
        account_id_map.insert(account_id.clone(), *account);
        account_id_list.push(account_id);
    }

    let sorted_list = DasSortedList::new(account_id_list);
    for item in sorted_list.items() {
        let account = account_id_map.get(item).unwrap();
        println!("{} => {}", account, item.pack());
    }
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
}

impl TemplateGenerator {
    pub fn new(action: &str, params_opt: Option<Bytes>) -> TemplateGenerator {
        let witness = das_util::wrap_action_witness(action, params_opt);

        TemplateGenerator {
            header_deps: Vec::new(),
            cell_deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            witnesses: vec![bytes_to_hex(witness)],
        }
    }

    fn push_value(&mut self, value: Value, group: Source) {
        match group {
            Source::HeaderDep => self.header_deps.push(value),
            Source::CellDep => self.cell_deps.push(value),
            Source::Input => self.inputs.push(value),
            Source::Output => self.outputs.push(value),
        }
    }

    pub fn gen_witness<T: Entity>(
        &mut self,
        data_type: DataType,
        output_opt: Option<(u32, u32, T)>,
        input_opt: Option<(u32, u32, T)>,
        dep_opt: Option<(u32, u32, T)>,
    ) {
        let witness = das_util::wrap_data_witness(data_type, output_opt, input_opt, dep_opt);
        self.witnesses.push(bytes_to_hex(witness));
    }

    pub fn gen_time_cell(&mut self, index: u8, timestamp: u64) {
        let index = Bytes::from(vec![index]);
        let timestamp = Uint64::from(timestamp);
        let raw = [
            index.as_reader().raw_data(),
            timestamp.as_reader().raw_data(),
        ]
        .concat();

        let cell_data = Bytes::from(raw);
        let data = json!({
          "tmp_type": "full",
          "capacity": 1000,
          "lock": {
            "code_hash": "{{always_success}}"
          },
          "type": {
            "code_hash": "0x0100000000000000000000000000000000000000000000000000000000000000",
            "hash_type": "type"
          },
          "tmp_data": bytes_to_hex(cell_data)
        });
        self.push_value(data, Source::CellDep);
    }

    pub fn gen_config_cell(&mut self, group: Source) -> ConfigCellData {
        let entity = ConfigCellData::new_builder()
            .reserved_account_filter(Bytes::default())
            .proposal_min_confirm_require(Uint8::from(4))
            .proposal_min_extend_interval(Uint8::from(2))
            .proposal_max_account_affect(Uint32::from(50))
            .proposal_max_pre_account_contain(Uint32::from(50))
            .apply_min_waiting_time(Uint32::from(60))
            .apply_max_waiting_time(Uint32::from(86400))
            .account_max_length(Uint32::from(1000))
            .price_configs(gen_price_config_list())
            .char_sets(gen_char_sets())
            .min_ttl(Uint32::from(300))
            .closing_limit_of_primary_market_auction(Uint32::from(86400))
            .closing_limit_of_secondary_market_auction(Uint32::from(86400))
            .type_id_table(gen_type_id_table())
            .build();

        // Generate the cell structure of ConfigCell.
        let cell_data = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let data = json!({
          "tmp_type": "full",
          "capacity": 1000,
          "lock": {
            "code_hash": "{{always_success}}",
            "args": "0x0000000000000000000000000000000000000000"
          },
          "type": {
            "code_hash": "{{config-cell-type}}"
          },
          "tmp_data": "0x".to_string() + &hex_string(cell_data.as_reader().raw_data()).unwrap()
        });
        self.push_value(data, group);

        entity
    }

    pub fn gen_apply_register_cell(
        &mut self,
        pubkey_hash: &str,
        account: &AccountChars,
        timestamp: u64,
        group: Source,
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

        // Generate the cell structure of ApplyRegisterCell.
        let cell_data = Bytes::from(raw);
        let data = json!({
          "tmp_type": "full",
          "capacity": 1000,
          "lock": {
            "code_hash": "{{always_success}}"
          },
          "type": {
            "code_hash": "{{apply-register-cell-type}}"
          },
          "tmp_data": bytes_to_hex(cell_data)
        });

        self.push_value(data, group);
    }

    pub fn gen_pre_account_cell(
        &mut self,
        account: &str,
        account_chars: &AccountChars,
        created_at: u64,
        group: Source,
    ) -> PreAccountCellData {
        let account_string = account.to_string() + ".bit";
        let id = util::account_to_id(account_string.as_bytes().to_vec());

        let entity = PreAccountCellData::new_builder()
            .account(account_chars.to_owned())
            .owner_lock(Script::default())
            .refund_lock(Script::default())
            .price(gen_price_config(8, 5_000_000, 500_000))
            .quote(Uint64::from(1_000))
            .created_at(Timestamp::from(created_at))
            .build();

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [hash.as_reader().raw_data(), id.as_slice()].concat();

        // Generate the cell structure of PreAccountCell.
        let cell_data = Bytes::from(raw);
        let data = json!({
          "tmp_type": "full",
          "capacity": 1000,
          "lock": {
            "code_hash": "{{always_success}}"
          },
          "type": {
            "code_hash": "{{pre-account-cell-type}}"
          },
          "tmp_data": bytes_to_hex(cell_data)
        });

        self.push_value(data, group);

        entity
    }

    pub fn gen_account_cell(
        &mut self,
        account: &str,
        account_chars: &AccountChars,
        next: bytes::Bytes,
        registered_at: u64,
        expired_at: u64,
        group: Source,
    ) -> AccountCellData {
        let account_string = account.to_string() + ".bit";
        let id = util::account_to_id(account_string.as_bytes().to_vec());

        let entity = AccountCellData::new_builder()
            .id(AccountId::try_from(id.clone()).unwrap())
            .account(account_chars.to_owned())
            .owner(Script::default())
            .manager(Script::default())
            .status(Uint8::from(0))
            .registered_at(Timestamp::from(registered_at))
            .expired_at(Timestamp::from(expired_at))
            .build();

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();

        let raw = [
            hash.as_reader().raw_data(),
            id.as_slice(),
            &next[..],
            Timestamp::from(expired_at).as_reader().raw_data(),
            account_string.as_bytes(),
        ]
        .concat();

        // Generate the cell structure of AccountCell.
        let cell_data = Bytes::from(raw);
        let data = json!({
          "tmp_type": "full",
          "capacity": 1000,
          "lock": {
            "code_hash": "{{always_success}}"
          },
          "type": {
            "code_hash": "{{account-cell-type}}"
          },
          "tmp_data": bytes_to_hex(cell_data)
        });

        self.push_value(data, group);

        entity
    }

    pub fn gen_slice_data_and_witness(
        &mut self,
        slices: &Vec<Vec<(&str, ProposalSliceItemType, &str, &str, u64, u64)>>,
        start_from: u32,
    ) {
        let mut dep_index = start_from;
        for slice in slices {
            for (_, item_type, next, account, registered_at, expired_at) in slice {
                let account_chars = gen_account_chars(account.split("").collect());
                if *item_type == ProposalSliceItemType::Exist {
                    let entity = self.gen_account_cell(
                        account,
                        &account_chars,
                        util::hex_to_bytes(next).unwrap(),
                        registered_at.to_owned(),
                        expired_at.to_owned(),
                        Source::CellDep,
                    );
                    self.gen_witness(
                        DataType::AccountCellData,
                        None,
                        None,
                        Some((1, dep_index, entity)),
                    );
                } else {
                    let entity = self.gen_pre_account_cell(
                        account,
                        &account_chars,
                        registered_at.to_owned(),
                        Source::CellDep,
                    );
                    self.gen_witness(
                        DataType::PreAccountCellData,
                        None,
                        None,
                        Some((1, dep_index, entity)),
                    );
                }

                dep_index += 1;
            }
        }
    }

    fn gen_proposal_item(
        &self,
        account_id: &str,
        item_type: &ProposalSliceItemType,
        next: &str,
    ) -> ProposalItem {
        let account_id =
            AccountId::try_from(util::hex_to_bytes(account_id).unwrap().to_vec()).unwrap();
        let mut builder = ProposalItem::new_builder()
            .account_id(account_id)
            .item_type(Uint8::from(*item_type as u8));

        if !next.is_empty() {
            let next_account_id =
                AccountId::try_from(util::hex_to_bytes(next).unwrap().to_vec()).unwrap();
            builder = builder.next(
                AccountIdOpt::new_builder()
                    .set(Some(next_account_id))
                    .build(),
            );
        }

        builder.build()
    }

    fn gen_slices(
        &self,
        slices: &Vec<Vec<(&str, ProposalSliceItemType, &str, &str, u64, u64)>>,
    ) -> SliceList {
        let mut sl_list = SliceList::new_builder();
        for slice in slices {
            if slice.len() <= 1 {
                panic!("Slice must has more than one item.")
            }

            let mut sl = SL::new_builder();
            let mut next_of_first_item = "";
            for (index, (account_id, item_type, next, _, _, _)) in slice.iter().enumerate() {
                // When it is the first item, saving its next.
                if index == 0 {
                    next_of_first_item = next;
                    let (next, _, _, _, _, _) = slice.get(index + 1).unwrap();
                    sl = sl.push(self.gen_proposal_item(account_id, item_type, next));
                // When it is the last item, use next_of_first_item as its next.
                } else if index == slice.len() - 1 {
                    sl = sl.push(self.gen_proposal_item(account_id, item_type, next_of_first_item));
                // When it is the items between the first and the last, using its next item's account_id as next.
                } else {
                    let (next, _, _, _, _, _) = slice.get(index + 1).unwrap();
                    sl = sl.push(self.gen_proposal_item(account_id, item_type, next));
                }
            }
            sl_list = sl_list.push(sl.build());
        }
        sl_list.build()
    }

    pub fn gen_proposal_cell(
        &mut self,
        slices: &Vec<Vec<(&str, ProposalSliceItemType, &str, &str, u64, u64)>>,
        group: Source,
    ) -> ProposalCellData {
        let entity = ProposalCellData::new_builder()
            .starter_lock(Script::default())
            .slices(self.gen_slices(slices))
            .build();

        // Generate the cell structure of ProposalCell.
        let cell_data = Bytes::from(&blake2b_256(entity.as_slice())[..]);
        let data = json!({
          "tmp_type": "full",
          "capacity": 1000,
          "lock": {
            "code_hash": "{{always_success}}"
          },
          "type": {
            "code_hash": "{{proposal-cell-type}}"
          },
          "tmp_data": bytes_to_hex(cell_data)
        });

        self.push_value(data, group);

        entity
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
