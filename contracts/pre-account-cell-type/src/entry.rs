use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::cmp::Ordering;
use core::convert::{TryFrom, TryInto};
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::*;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, code_to_error, data_parser, debug, util, verifiers, warn};
use das_sorted_list::util as sorted_list_util;
use das_types::constants::*;
use das_types::mixer::PreAccountCellDataReaderMixer;
use das_types::packed::*;
use das_types::prelude::*;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running pre-account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );

    match action {
        b"confirm_proposal" => {
            util::require_type_script(
                &parser,
                TypeScript::ProposalCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"pre_register" => {
            debug!("Find out PreAccountCell ...");

            parser.parse_cell()?;
            let config_main_reader = parser.configs.main()?;

            debug!("Parse cells in transaction ...");

            let dep_account_cells = util::find_cells_by_type_id(
                ScriptType::Type,
                config_main_reader.type_id_table().account_cell(),
                Source::CellDep
            )?;

            verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

            let (input_apply_register_cells, output_apply_register_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(
                    ScriptType::Type,
                    config_main_reader.type_id_table().apply_register_cell(),
                )?;

            verifiers::common::verify_cell_number(
                "ApplyRegisterCell",
                &input_apply_register_cells,
                1,
                &output_apply_register_cells,
                0,
            )?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("PreRegisterCell", &input_cells, 0, &output_cells, 1)?;

            debug!("Read data of ApplyRegisterCell ...");

            let config_apply_reader = parser.configs.apply()?;
            let height = util::load_oracle_data(OracleCellType::Height)?;

            // Read the hash from outputs_data of the ApplyRegisterCell.
            let index = &input_apply_register_cells[0];
            let apply_register_lock = high_level::load_cell_lock(index.to_owned(), Source::Input)?;
            let data = high_level::load_cell_data(index.to_owned(), Source::Input)?;
            let apply_register_hash = data_parser::apply_register_cell::get_account_hash(&data)?;

            verify_apply_height(height, config_apply_reader, &data)?;

            debug!("Read witness of PreAccountCell ...");

            // Read outputs_data and witness of the PreAccountCell.
            let data = high_level::load_cell_data(output_cells[0], Source::Output)?;
            let account_id = data_parser::pre_account_cell::get_id(&data);
            let capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;

            let pre_account_cell_witness =
                util::parse_pre_account_cell_witness(&parser, output_cells[0], Source::Output)?;
            let pre_account_cell_witness_reader = pre_account_cell_witness.as_reader();

            debug!("Verify various fields of PreAccountCell ...");

            let config_price = parser.configs.price()?;
            let config_account = parser.configs.account()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            verifiers::misc::verify_always_success_lock(output_cells[0], Source::Output)?;
            verify_apply_hash(
                &pre_account_cell_witness_reader,
                apply_register_lock.as_reader().args().raw_data().to_vec(),
                &apply_register_hash,
            )?;
            verify_owner_lock_args(&pre_account_cell_witness_reader)?;
            verify_quote(&pre_account_cell_witness_reader)?;
            verify_invited_discount(config_price, &pre_account_cell_witness_reader)?;
            verify_price_and_capacity(config_account, config_price, &pre_account_cell_witness_reader, capacity)?;
            verify_account_id(&pre_account_cell_witness_reader, account_id)?;
            verify_created_at(timestamp, &pre_account_cell_witness_reader)?;
            verify_account_not_exist(dep_account_cells[0], account_id)?;

            debug!("Verify if account is available for registration for now ...");

            let cells_with_super_lock =
                util::find_cells_by_script(ScriptType::Lock, super_lock().as_reader(), Source::Input)?;

            match verify_account_length_and_years(&pre_account_cell_witness_reader, timestamp) {
                Ok(_) => {}
                Err(err) => {
                    if err.as_i8() == ErrorCode::AccountStillCanNotBeRegister as i8 && cells_with_super_lock.len() > 0 {
                        debug!("Skip ErrorCode::AccountStillCanNotBeRegister because of super lock.");
                        // Ok
                    } else {
                        return Err(err);
                    }
                }
            }

            let config_release = parser.configs.release()?;
            match verify_account_release_status(timestamp, config_release, &pre_account_cell_witness_reader) {
                Ok(_) => {}
                Err(err) => {
                    if err.as_i8() == ErrorCode::AccountStillCanNotBeRegister as i8 && cells_with_super_lock.len() > 0 {
                        debug!("Skip ErrorCode::AccountStillCanNotBeRegister because of super lock.");
                        // Ok
                    } else {
                        return Err(err);
                    }
                }
            }

            let account = pre_account_cell_witness_reader.account().as_readable();
            match verifiers::account_cell::verify_preserved_accounts(&parser, &account) {
                Ok(_) => {}
                Err(err) => {
                    if err.as_i8() == ErrorCode::AccountIsPreserved as i8 && cells_with_super_lock.len() > 0 {
                        debug!("Skip ErrorCode::AccountIsPreserved because of super lock.");
                        // Ok
                    } else {
                        return Err(err);
                    }
                }
            }
            verifiers::account_cell::verify_unavailable_accounts(&parser, &account)?;

            let chars_reader = pre_account_cell_witness_reader.account();
            verifiers::account_cell::verify_account_chars(&parser, chars_reader)?;
            verifiers::account_cell::verify_account_chars_max_length(&parser, chars_reader)?;

            match pre_account_cell_witness_reader.version() {
                2 => {
                    if let Ok(reader) = pre_account_cell_witness_reader.try_into_v2() {
                        verifiers::account_cell::verify_records_keys(&parser, reader.initial_records())?;
                    } else {
                        warn!("The PreAccountCellDataReaderMixer.version returned a mismatched version number.");
                        return Err(code_to_error!(ErrorCode::HardCodedError));
                    }
                }
                3 => {
                    if let Ok(reader) = pre_account_cell_witness_reader.try_into_latest() {
                        verifiers::account_cell::verify_records_keys(&parser, reader.initial_records())?;
                    } else {
                        warn!("The PreAccountCellDataReaderMixer.version returned a mismatched version number.");
                        return Err(code_to_error!(ErrorCode::HardCodedError));
                    }
                }
                _ => {}
            }
        }
        b"refund_pre_register" => {
            parser.parse_cell()?;
            let config_main_reader = parser.configs.main()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;
            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            assert!(
                input_cells.len() > 0 && output_cells.len() == 0,
                ErrorCode::InvalidTransactionStructure,
                "There should be at least 1 PreAccountCell in inputs and none in outputs.(in_inputs: {}, in_outputs: {})",
                input_cells.len(),
                output_cells.len()
            );

            debug!("Collect the capacities of all PreAccountCells ...");

            let mut refund_map = BTreeMap::new();
            for index in input_cells {
                let pre_account_cell_witness = util::parse_pre_account_cell_witness(&parser, index, Source::Input)?;
                let pre_account_cell_witness_reader = pre_account_cell_witness.as_reader();
                let capacity = high_level::load_cell_capacity(index, Source::Input)?;
                let created_at = u64::from(pre_account_cell_witness_reader.created_at());

                assert!(
                    timestamp >= created_at + PRE_ACCOUNT_CELL_TIMEOUT,
                    PreAccountCellErrorCode::PreAccountCellIsNotTimeout,
                    "The PreAccountCell is not timeout, so it can not be refunded for now.(current: {}, created_at: {}, timeout_limit: {})",
                    timestamp,
                    created_at,
                    PRE_ACCOUNT_CELL_TIMEOUT
                );

                util::map_add(
                    &mut refund_map,
                    pre_account_cell_witness_reader.refund_lock().as_slice().to_vec(),
                    capacity,
                );
            }

            debug!("Verify if every refund lock get its capacity properly ...");

            for (lock_bytes, &expect_capacity) in refund_map.iter() {
                let lock_reader = ScriptReader::from_slice(lock_bytes).unwrap();
                let cells = util::find_cells_by_script(ScriptType::Lock, lock_reader.into(), Source::Output)?;

                assert!(
                    cells.len() == 1,
                    ErrorCode::InvalidTransactionStructure,
                    "There should be only 1 cell to take the refund.(expected: 1, result: {})",
                    cells.len()
                );

                let current_capacity = high_level::load_cell_capacity(cells[0], Source::Output)?;

                assert!(
                    expect_capacity <= current_capacity + 10000,
                    PreAccountCellErrorCode::RefundCapacityError,
                    "The refund of PreAccountCell to {} should be {} shannon.(expected: {}, result: {})",
                    lock_reader.args(),
                    expect_capacity,
                    expect_capacity,
                    current_capacity
                );
            }

            verifiers::balance_cell::verify_das_lock_always_with_type(config_main_reader)?;
        }
        _ => {
            return Err(code_to_error!(ErrorCode::ActionNotSupported));
        }
    }

    Ok(())
}

fn verify_apply_height(
    current_height: u64,
    config_reader: ConfigCellApplyReader,
    data: &[u8],
) -> Result<(), Box<dyn ScriptError>> {
    // Read the apply timestamp from outputs_data of ApplyRegisterCell.
    let apply_height = data_parser::apply_register_cell::get_height(data);

    // Check that the ApplyRegisterCell has existed long enough, but has not yet timed out.
    let apply_min_waiting_block = u32::from(config_reader.apply_min_waiting_block_number());
    let apply_max_waiting_block = u32::from(config_reader.apply_max_waiting_block_number());
    let passed_block_number = if current_height > apply_height {
        current_height - apply_height
    } else {
        0
    };

    debug!(
        "Has passed {} block after apply.(min waiting: {} block, max waiting: {} block)",
        passed_block_number, apply_min_waiting_block, apply_max_waiting_block
    );

    assert!(
        passed_block_number >= apply_min_waiting_block as u64,
        ErrorCode::ApplyRegisterNeedWaitLonger,
        "The ApplyRegisterCell need to wait longer.(passed: {}, min_wait: {})",
        passed_block_number,
        apply_min_waiting_block
    );
    assert!(
        passed_block_number <= apply_max_waiting_block as u64,
        ErrorCode::ApplyRegisterHasTimeout,
        "The ApplyRegisterCell has been timeout.(passed: {}, max_wait: {})",
        passed_block_number,
        apply_max_waiting_block
    );

    Ok(())
}

fn verify_account_id<'a>(
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    account_id: &[u8],
) -> Result<(), Box<dyn ScriptError>> {
    let account: Vec<u8> = [reader.account().as_readable(), ACCOUNT_SUFFIX.as_bytes().to_vec()].concat();
    let expected_account_id = util::get_account_id_from_account(&account);

    assert!(
        &expected_account_id == account_id,
        PreAccountCellErrorCode::AccountIdIsInvalid,
        "PreAccountCell.account_id should be calculated from account correctly.(account: {:?}, expected_account_id: 0x{})",
        String::from_utf8(account),
        util::hex_string(&expected_account_id)
    );

    Ok(())
}

fn verify_apply_hash<'a>(
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    apply_register_cell_lock_args: Vec<u8>,
    current_hash: &[u8],
) -> Result<(), Box<dyn ScriptError>> {
    let data_to_hash: Vec<u8> = [
        apply_register_cell_lock_args.clone(),
        reader.account().as_readable(),
        ".bit".as_bytes().to_vec(),
    ]
    .concat();
    let expected_hash = util::blake2b_256(data_to_hash.as_slice());

    assert!(
        current_hash == expected_hash,
        PreAccountCellErrorCode::ApplyHashMismatch,
        "The hash in ApplyRegisterCell should be calculated from blake2b(ApplyRegisterCell.lock.args + account).(expected: 0x{}, current: 0x{})",
        util::hex_string(&expected_hash),
        util::hex_string(current_hash)
    );

    Ok(())
}

fn verify_created_at<'a>(
    expected_timestamp: u64,
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    let create_at = u64::from(reader.created_at());

    assert!(
        create_at == expected_timestamp,
        PreAccountCellErrorCode::CreateAtIsInvalid,
        "PreAccountCell.created_at should be the same as the TimeCell.(expected: {}, current: {})",
        expected_timestamp,
        create_at
    );

    Ok(())
}

fn verify_owner_lock_args<'a>(
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if PreAccountCell.witness.owner_lock_args is more than 1 byte and the first byte is 0x00.");

    let owner_lock_args = reader.owner_lock_args().raw_data();

    assert!(
        owner_lock_args.len() >= 42,
        PreAccountCellErrorCode::OwnerLockArgsIsInvalid,
        "The length of owner_lock_args should be more 42 byte, but {} found.",
        owner_lock_args.len()
    );

    Ok(())
}

fn verify_quote<'a>(reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if PreAccountCell.witness.quote is the same as QuoteCell.");

    let expected_quote = util::load_oracle_data(OracleCellType::Quote)?;
    let current = u64::from(reader.quote());

    assert!(
        expected_quote == current,
        PreAccountCellErrorCode::QuoteIsInvalid,
        "PreAccountCell.quote should be the same as the QuoteCell.(expected: {:?}, current: {:?})",
        expected_quote,
        current
    );

    Ok(())
}

fn verify_invited_discount<'a>(
    config: ConfigCellPriceReader,
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if PreAccountCell.witness.invited_discount is 0 or the same as configuration.");

    let default_lock = Script::default();
    let default_lock_reader = default_lock.as_reader();

    let zero = Uint32::from(0);
    let expected_discount;

    if reader.inviter_lock().is_none() {
        assert!(
            reader.inviter_id().is_empty(),
            PreAccountCellErrorCode::InviterIdShouldBeEmpty,
            "The inviter_id should be empty when inviter do not exist."
        );

        expected_discount = zero.as_reader();
        assert!(
            util::is_reader_eq(expected_discount, reader.invited_discount()),
            PreAccountCellErrorCode::InviteeDiscountShouldBeEmpty,
            "The invited_discount should be 0 when inviter does not exist."
        );
    } else {
        let inviter_lock_reader = reader.inviter_lock().to_opt().unwrap();
        // Skip default value for supporting transactions treat default value as None.
        if util::is_reader_eq(default_lock_reader, inviter_lock_reader) {
            assert!(
                reader.inviter_id().is_empty(),
                PreAccountCellErrorCode::InviterIdShouldBeEmpty,
                "The inviter_id should be empty when inviter do not exist."
            );

            expected_discount = zero.as_reader();
            assert!(
                util::is_reader_eq(expected_discount, reader.invited_discount()),
                PreAccountCellErrorCode::InviteeDiscountShouldBeEmpty,
                "The invited_discount should be 0 when inviter does not exist."
            );
        } else {
            assert!(
                reader.inviter_id().len() == ACCOUNT_ID_LENGTH,
                PreAccountCellErrorCode::InviterIdIsInvalid,
                "The inviter_id should be 20 bytes when inviter exists."
            );

            expected_discount = config.discount().invited_discount();
            assert!(
                util::is_reader_eq(expected_discount, reader.invited_discount()),
                PreAccountCellErrorCode::InviteeDiscountIsInvalid,
                "The invited_discount should greater than 0 when inviter exist. (expected: {}, current: {})",
                u32::from(expected_discount),
                u32::from(reader.invited_discount())
            );
        }
    }

    Ok(())
}

fn verify_price_and_capacity<'a>(
    config_account: ConfigCellAccountReader,
    config_price: ConfigCellPriceReader,
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    capacity: u64,
) -> Result<(), Box<dyn ScriptError>> {
    let length_in_price = util::get_length_in_price(reader.account().len() as u64);
    let price = reader.price();
    let prices = config_price.prices();

    // Find out register price in from ConfigCellRegister.
    let expected_price = prices
        .iter()
        .find(|item| u8::from(item.length()) == length_in_price)
        .ok_or(ErrorCode::ItemMissing)?;

    debug!("Check if PreAccountCell.witness.price is selected base on account length.");

    assert!(
        util::is_reader_eq(expected_price, price),
        PreAccountCellErrorCode::PriceIsInvalid,
        "PreAccountCell.price should be the same as which in ConfigCellPrice.(expected: {}, current: {})",
        expected_price,
        price
    );

    let new_account_price_in_usd = u64::from(reader.price().new()); // x USD
    let discount = u32::from(reader.invited_discount());
    let quote = u64::from(reader.quote()); // y CKB/USD

    // Register price for 1 year in CKB = x ÷ y.
    let register_capacity = util::calc_yearly_capacity(new_account_price_in_usd, quote, discount);
    // Storage price in CKB = AccountCell base capacity + account.bytes.length
    let storage_capacity = util::calc_account_storage_capacity(
        config_account,
        reader.account().as_readable().len() as u64 + 4,
        reader.owner_lock_args(),
    );

    debug!("Check if PreAccountCell.capacity is enough for registration: {}(paid) <-> {}(1 year registeration fee) + {}(storage fee)",
        capacity,
        register_capacity,
        storage_capacity
    );

    assert!(
        capacity >= register_capacity + storage_capacity,
        PreAccountCellErrorCode::CKBIsInsufficient,
        "PreAccountCell.capacity should contains more than 1 year of registeration fee. (expected: {}, current: {})",
        register_capacity + storage_capacity,
        capacity
    );

    Ok(())
}

fn verify_account_length_and_years<'a>(
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
    current_timestamp: u64,
) -> Result<(), Box<dyn ScriptError>> {
    use chrono::{DateTime, NaiveDateTime, Utc};

    let account_length = reader.account().len();
    let _current = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(current_timestamp as i64, 0), Utc);

    debug!(
        "Check if the account is available for registration now. (length: {}, current: {:#?})",
        account_length, _current
    );

    // On CKB main net, AKA Lina, accounts of less lengths can be registered only after a specific number of years.
    // CAREFUL Triple check.
    assert!(
        account_length >= 4,
        ErrorCode::AccountStillCanNotBeRegister,
        "The account less than 4 characters can not be registered now."
    );

    Ok(())
}

fn verify_account_release_status<'a>(
    timestamp: u64,
    config_release: ConfigCellReleaseReader,
    reader: &Box<dyn PreAccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if account is released for registration.");

    if reader.account().len() >= 10 {
        debug!("Ths account contains more than 9 characters, skip verification.");
        return Ok(());
    }

    // 1666094400 is 2022-10-18 12:00:00 UTC.
    if timestamp >= 1666094400 {
        debug!("It is after 2022-10-18 12:00:00 UTC now, start fully released char-sets verification.");

        let account_chars = reader.account();
        // Only if all characters are the same char-set, the account_char_set will have value.
        let mut account_char_set = None;
        for char in account_chars.iter() {
            let char_set = CharSetType::try_from(char.char_set_name()).map_err(|_| ErrorCode::CharSetIsUndefined)?;
            if account_char_set.is_none() {
                account_char_set = Some(char_set);
            } else if account_char_set != Some(char_set) {
                account_char_set = None;
                break;
            }
        }

        debug!("The account_char_set is: {:?}", account_char_set);

        let completely_released_char_set = vec![
            CharSetType::Emoji,
            CharSetType::Digit,
            CharSetType::Ko,
            CharSetType::Th,
        ];
        if let Some(char_set) = account_char_set {
            // If the account_char_set is in while list and the account's length is greater than or equel to 4, then the account is released.
            if account_chars.len() >= 4 && completely_released_char_set.contains(&char_set) {
                debug!(
                    "The account contains only one fully released char-set: {:?}, skip verification.",
                    char_set
                );
                return Ok(());
            } else {
                debug!("The account is not fullfilled the fully released rule, continue the next verification.");
            }
        }
    }

    let account: Vec<u8> = [reader.account().as_readable(), ACCOUNT_SUFFIX.as_bytes().to_vec()].concat();
    let hash = util::blake2b_das(account.as_slice());
    let lucky_num = u32::from_be_bytes((&hash[0..4]).try_into().unwrap());
    let expected_lucky_num = u32::from(config_release.lucky_number());

    // CAREFUL Triple check.
    assert!(
        lucky_num <= expected_lucky_num,
        ErrorCode::AccountStillCanNotBeRegister,
        "The registration is still not started.(lucky_num: {}, required: <= {})",
        lucky_num,
        expected_lucky_num
    );

    debug!(
        "The account has been released.(lucky_num: {}, required: <= {})",
        lucky_num, expected_lucky_num
    );

    Ok(())
}

fn verify_account_not_exist(pre_account_cell: usize, account_id: &[u8]) -> Result<(), Box<dyn ScriptError>> {
    let account_data = high_level::load_cell_data(pre_account_cell, Source::CellDep)?;
    let pre_account_id = data_parser::account_cell::get_id(&account_data);
    let pre_account_next = data_parser::account_cell::get_next(&account_data);

    assert!(
        sorted_list_util::cmp(pre_account_id, account_id) == Ordering::Greater &&
        sorted_list_util::cmp(account_id, pre_account_next) == Ordering::Greater,
        PreAccountCellErrorCode::AccountAlreadyExistOrProofInvalid,
        "The account already exists or the proof is invalid.(0x{} > 0x{} > 0x{})",
        util::hex_string(pre_account_id),
        util::hex_string(account_id),
        util::hex_string(pre_account_next)
    );

    Ok(())
}
