use das_types::constants::*;
use das_types::packed::*;
use molecule::prelude::*;

fn gen_account_char(char: &str, char_set_type: CharSetType) -> AccountChar {
    AccountChar::new_builder()
        .char_set_name(Uint32::from(char_set_type as u32))
        .bytes(Bytes::from(char.as_bytes()))
        .build()
}

#[test]
fn test_account_chars_support_as_readable() {
    // Convert from Hash between Vec
    let expected = "das✨";
    let account_chars = AccountChars::new_builder()
        .push(gen_account_char("d", CharSetType::En))
        .push(gen_account_char("a", CharSetType::En))
        .push(gen_account_char("s", CharSetType::En))
        .push(gen_account_char("✨", CharSetType::En))
        .build();

    assert_eq!(account_chars.as_readable().as_slice(), expected.as_bytes());
    assert_eq!(account_chars.as_reader().as_readable().as_slice(), expected.as_bytes());
}
