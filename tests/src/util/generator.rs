use super::util;
use ckb_tool::{ckb_hash::blake2b_256, ckb_types::bytes};
use das_types::{constants::*, packed::*, prelude::*};
use std::convert::TryFrom;

fn gen_price_config(length: u8, new_price: u64, renew_price: u64) -> PriceConfig {
    PriceConfig::new_builder()
        .length(Uint8::from(length))
        .new(Uint64::from(new_price))
        .renew(Uint64::from(renew_price))
        .build()
}

pub fn gen_price_config_list() -> PriceConfigList {
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

pub fn gen_char_sets() -> CharSetList {
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

pub fn gen_type_id_table() -> TypeIdTable {
    TypeIdTable::new_builder()
        .apply_register_cell(
            util::hex_to_hash("0xcac501b0a5826bffa485ccac13c2195fcdf3aa86b113203f620ddd34d3decd70")
                .unwrap(),
        )
        .pre_account_cell(
            util::hex_to_hash("0x431a3af2d4bbcd69ab732d37be794ac0ab172c151545dfdbae1f578a7083bc84")
                .unwrap(),
        )
        .build()
}

pub fn gen_config_cell_data() -> ConfigCellData {
    let config_cell_data = ConfigCellData::new_builder()
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

    let hash_of_config_cell_data =
        Hash::try_from(blake2b_256(config_cell_data.as_slice()).to_vec()).unwrap();

    println!(
        "hash_of_config_cell_data(no header) = {}",
        hash_of_config_cell_data
    );

    config_cell_data
}

pub fn gen_time_cell_data(index: u8, timestamp: u64) -> Bytes {
    let index = Bytes::from(vec![index]);
    let timestamp = Uint64::from(timestamp);
    let raw = [
        index.as_reader().raw_data(),
        timestamp.as_reader().raw_data(),
    ]
    .concat();
    let time_cell_data = Bytes::from(raw);

    println!("data_of_time_cell(no header) = {}", time_cell_data);

    time_cell_data
}

pub fn gen_apply_register_cell_data(
    pubkey_hash: &str,
    account: &AccountChars,
    timestamp: u64,
) -> Bytes {
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
    let data = Bytes::from(raw);

    // println!("hash_of_account(no header) = {}", hash_of_account);
    // println!("timestamp(no header) = {}", Uint64::from(timestamp));
    println!("apply_register_cell_data = {}", data);

    data
}

pub fn gen_pre_account_cell_data(account: &AccountChars, created_at: u64) -> PreAccountCellData {
    let mut account_bytes = account.as_readable();
    account_bytes.append(&mut ".bit".as_bytes().to_vec());
    let entity = PreAccountCellData::new_builder()
        .account(account.to_owned())
        .owner_lock(Script::default())
        .refund_lock(Script::default())
        .price(gen_price_config(8, 5_000_000, 500_000))
        .quote(Uint64::from(1_000))
        .created_at(Timestamp::from(created_at))
        .build();

    let id = util::account_to_id(Bytes::from(account_bytes));
    // println!("account_id(no header) = {}", id);
    let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
    // println!("hash_of_pre_account_cell_data(no header) = {}", hash);

    let raw = [id.as_reader().raw_data(), hash.as_reader().raw_data()].concat();
    let pre_account_cell_data = Bytes::from(raw);
    println!("pre_account_cell_data = {}", pre_account_cell_data);

    entity
}

pub fn gen_proposal_cell_data() {}
