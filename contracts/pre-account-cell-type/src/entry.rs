use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock, load_script},
};
use core::convert::TryInto;
use core::result::Result;
use das_bloom_filter::BloomFilter;
use das_core::{assert, constants::*, debug, error::Error, util};
use das_types::{
    constants::{ConfigID, DataType},
    packed::*,
    prelude::*,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running pre-account-cell-type ======");

    let action_data = util::load_das_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        let mut parser = util::load_das_witnesses(Some(vec![DataType::ConfigCellMain]))?;
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"pre_register" {
        debug!("Route to pre_register action ...");

        debug!("Find out PreAccountCell ...");

        // Find out PreAccountCells in current transaction.
        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        // Consuming PreAccountCell is not allowed in pre_register action.
        if old_cells.len() != 0 {
            return Err(Error::PreRegisterFoundInvalidTransaction);
        }
        // Only one PreAccountCell can be created at one time.
        if new_cells.len() != 1 {
            return Err(Error::PreRegisterFoundInvalidTransaction);
        }

        debug!("Find out ApplyRegisterCell ...");

        let mut parser = util::load_das_witnesses(None)?;
        parser.parse_all_data()?;
        parser.parse_only_config(&[
            ConfigID::ConfigCellMain,
            ConfigID::ConfigCellRegister,
            ConfigID::ConfigCellBloomFilter,
        ])?;
        let configs = parser.configs();
        let config_main_reader = configs.main()?;

        let old_apply_register_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config_main_reader.type_id_table().apply_register_cell(),
            Source::Input,
        )?;
        let new_apply_register_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config_main_reader.type_id_table().apply_register_cell(),
            Source::Output,
        )?;

        // There must be one ApplyRegisterCell in inputs.
        if old_apply_register_cells.len() != 1 {
            return Err(Error::PreRegisterFoundInvalidTransaction);
        }
        // Creating ApplyRegisterCell is not allowed in this action.
        if new_apply_register_cells.len() != 0 {
            return Err(Error::PreRegisterFoundInvalidTransaction);
        }

        debug!("Read data of ApplyRegisterCell ...");

        // Read the hash from outputs_data of the ApplyRegisterCell.
        let index = &old_apply_register_cells[0];
        let data = load_cell_data(index.to_owned(), Source::Input).map_err(|e| Error::from(e))?;
        let apply_register_hash = match data.get(..32) {
            Some(bytes) => bytes,
            _ => return Err(Error::InvalidCellData),
        };
        let apply_register_lock =
            load_cell_lock(index.to_owned(), Source::Input).map_err(|e| Error::from(e))?;

        #[cfg(not(feature = "mainnet"))]
        das_core::inspect::apply_register_cell(Source::Input, index.to_owned(), &data);

        let height = util::load_height()?;
        let config_register_reader = configs.register()?;
        verify_apply_height(height, config_register_reader, &data)?;

        debug!("Read witness of PreAccountCell ...");

        // Read outputs_data and witness of the PreAccountCell.
        let index = &new_cells[0];
        let data = load_cell_data(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;
        let account_id = match data.get(32..) {
            Some(bytes) => Bytes::from(bytes),
            _ => return Err(Error::InvalidCellData),
        };
        let capacity =
            load_cell_capacity(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;
        let (_, _, entity) = parser.verify_and_get(index.to_owned(), Source::Output)?;

        #[cfg(not(feature = "mainnet"))]
        das_core::inspect::pre_account_cell(
            Source::Output,
            index.to_owned(),
            &data,
            entity.to_owned(),
        );

        let pre_account_witness = PreAccountCellData::from_slice(entity.as_reader().raw_data())
            .map_err(|_| Error::WitnessEntityDecodingError)?;
        let reader = pre_account_witness.as_reader();

        verify_apply_hash(
            reader,
            apply_register_lock.as_reader().args().raw_data().to_vec(),
            apply_register_hash,
        )?;

        verify_quote(reader)?;
        verify_invited_discount(config_register_reader, reader)?;
        verify_price_and_capacity(config_register_reader, reader, capacity)?;

        verify_account_id(reader, account_id.as_reader())?;

        let timestamp = util::load_timestamp()?;
        verify_created_at(timestamp, reader)?;
        util::verify_account_length_and_years(reader.account().len(), timestamp, None)?;

        verify_account_chars(config_register_reader, reader)?;

        let config_bloom_filter = configs.bloom_filter()?;
        verify_preserved_accounts(config_bloom_filter, reader)?;
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}

fn verify_apply_height(
    current_height: u64,
    config_reader: ConfigCellRegisterReader,
    data: &[u8],
) -> Result<(), Error> {
    // Read the apply timestamp from outputs_data of ApplyRegisterCell.
    let apply_height = match data.get(32..) {
        Some(bytes) => {
            if bytes.len() != 8 {
                return Err(Error::InvalidCellData);
            }
            u64::from_le_bytes(bytes.try_into().unwrap())
        }
        _ => return Err(Error::InvalidCellData),
    };

    // Check that the ApplyRegisterCell has existed long enough, but has not yet timed out.
    let apply_min_waiting_time = u32::from(config_reader.apply_min_waiting_block_number());
    let apply_max_waiting_time = u32::from(config_reader.apply_max_waiting_block_number());
    let passed_block_number = current_height - apply_height;

    debug!(
        "Has passed {} block after apply.(min waiting: {} block, max waiting: {} block)",
        passed_block_number, apply_min_waiting_time, apply_max_waiting_time
    );

    if passed_block_number < apply_min_waiting_time as u64 {
        return Err(Error::ApplyRegisterNeedWaitLonger);
    }
    if passed_block_number > apply_max_waiting_time as u64 {
        return Err(Error::ApplyRegisterHasTimeout);
    }

    Ok(())
}

fn verify_account_id(
    reader: PreAccountCellDataReader,
    account_id: BytesReader,
) -> Result<(), Error> {
    let account: Vec<u8> = [reader.account().as_readable(), ".bit".as_bytes().to_vec()].concat();
    let hash = util::blake2b_256(account.as_slice());

    debug!(
        "Verify account ID in PreAccountCell: hash_from({:?}){:?} != PreAccountCell.data.account_id{:?} {}",
        account,
        &hash[..10],
        account_id.raw_data(),
        &hash[..10] != account_id.raw_data()
    );

    // The account ID in PreAccountCell must be calculated from the account.
    if &hash[..10] != account_id.raw_data() {
        return Err(Error::PreRegisterAccountIdIsInvalid);
    }

    Ok(())
}

fn verify_apply_hash(
    reader: PreAccountCellDataReader,
    pubkey_hash: Vec<u8>,
    expected_hash: &[u8],
) -> Result<(), Error> {
    let data_to_hash: Vec<u8> = [
        pubkey_hash,
        reader.account().as_readable(),
        ".bit".as_bytes().to_vec(),
    ]
    .concat();
    let hash = util::blake2b_256(data_to_hash.as_slice());

    debug!(
        "Verify hash in ApplyRegisterCell: 0x{}(expected) != 0x{}(apply_register_cell.data)",
        util::hex_string(expected_hash),
        util::hex_string(&hash)
    );

    if expected_hash != hash {
        debug!(
            "Hash calculated from: arg: 0x{}, account: 0x{}",
            util::hex_string(data_to_hash.get(..20).unwrap()),
            util::hex_string(data_to_hash.get(20..).unwrap())
        );
        return Err(Error::PreRegisterApplyHashIsInvalid);
    }

    Ok(())
}

fn verify_created_at(
    current_timestamp: u64,
    reader: PreAccountCellDataReader,
) -> Result<(), Error> {
    let create_at = reader.created_at();
    if u64::from(create_at) != current_timestamp {
        return Err(Error::PreRegisterCreateAtIsInvalid);
    }

    Ok(())
}

fn verify_quote(reader: PreAccountCellDataReader) -> Result<(), Error> {
    let expected_quote = util::load_quote()?.to_le_bytes();

    if &expected_quote != reader.quote().raw_data() {
        return Err(Error::PreRegisterQuoteIsInvalid);
    }

    Ok(())
}

fn verify_invited_discount(
    config: ConfigCellRegisterReader,
    reader: PreAccountCellDataReader,
) -> Result<(), Error> {
    debug!("Check if PreAccountCell.witness.invited_discount is 0 or the same as configuration.");

    let zero = Uint32::from(0);
    let expected_discount;
    if reader.inviter_wallet().is_empty() {
        expected_discount = zero.as_reader();
        assert!(
            util::is_reader_eq(expected_discount, reader.invited_discount()),
            Error::PreRegisterDiscountIsInvalid,
            "The invited_discount should be 0 when inviter do not exist."
        );
    } else {
        expected_discount = config.discount().invited_discount();
        assert!(
            util::is_reader_eq(expected_discount, reader.invited_discount()),
            Error::PreRegisterDiscountIsInvalid,
            "The invited_discount should greater than 0 when inviter exist. (expected: {}, current: {})",
            u32::from(expected_discount),
            u32::from(reader.invited_discount())
        );
    }

    Ok(())
}

fn verify_price_and_capacity(
    config: ConfigCellRegisterReader,
    reader: PreAccountCellDataReader,
    capacity: u64,
) -> Result<(), Error> {
    let length_in_price = util::get_length_in_price(reader.account().len() as u64);
    let price = reader.price();
    let prices = config.price_configs();

    // Find out register price in from ConfigCellRegister.
    let mut price_opt = Some(prices.get(prices.len() - 1).unwrap());
    for item in prices.iter() {
        if u8::from(item.length()) == length_in_price {
            price_opt = Some(item);
            break;
        }
    }
    let expected_price = price_opt.unwrap(); // x USD

    debug!("Check if PreAccountCell.witness.price is selected base on account length.");

    if !util::is_reader_eq(expected_price, price) {
        debug!(
            "PreAccountCell.price is invalid: {}(expected.length) != {}(result.length)",
            u8::from(reader.price().length()),
            u8::from(expected_price.length())
        );
        return Err(Error::PreRegisterPriceInvalid);
    }

    let new_account_price_in_usd = u64::from(reader.price().new()); // x USD
    let discount = u32::from(reader.invited_discount());
    let quote = u64::from(reader.quote()); // y CKB/USD

    // Register price for 1 year in CKB = x ÷ y.
    let register_capacity = util::calc_yearly_capacity(new_account_price_in_usd, quote, discount);
    // Storage price in CKB = AccountCell base capacity + RefCell base capacity + account.length
    let storage_capacity = util::calc_account_storage_capacity(reader.account().len() as u64 + 4);

    debug!("Check if PreAccountCell.capacity is enough for registration: {}(paid) < {}(1 year registeration fee) + {}(storage fee)",
        capacity,
        register_capacity,
        storage_capacity
    );

    assert!(
        capacity >= register_capacity + storage_capacity,
        Error::PreRegisterCKBInsufficient,
        "PreAccountCell.capacity should contains more than 1 year of registeration fee. (expected: {}, current: {})",
        register_capacity + storage_capacity,
        capacity
    );

    Ok(())
}

fn verify_account_chars(
    config: ConfigCellRegisterReader,
    reader: PreAccountCellDataReader,
) -> Result<(), Error> {
    debug!("Verify if account chars is available.");

    let char_set_list = config.char_sets();
    let mut prev_char_set_name: Option<_> = None;
    for account_char in reader.account().iter() {
        let char_set_opt = char_set_list
            .iter()
            .find(|char_set| util::is_reader_eq(char_set.name(), account_char.char_set_name()));
        match char_set_opt {
            Some(char_set) => {
                // Store the first non-global char set by default.
                if u8::from(char_set.global()) == 0 {
                    if prev_char_set_name.is_none() {
                        prev_char_set_name = Some(char_set.name());
                    } else {
                        // No other character set can be different from the first one.
                        if !util::is_reader_eq(prev_char_set_name.unwrap(), char_set.name()) {
                            return Err(Error::PreRegisterAccountCharSetConflict);
                        }
                    }
                }

                // Check if the char is in the char set.
                let is_char_valid = char_set
                    .chars()
                    .iter()
                    .any(|char| util::is_reader_eq(account_char.bytes(), char));
                if !is_char_valid {
                    debug!(
                        "The invalid char is: {:x?}",
                        account_char.bytes().raw_data()
                    );

                    return Err(Error::PreRegisterAccountCharIsInvalid);
                }
            }
            _ => return Err(Error::PreRegisterFoundUndefinedCharSet),
        }
    }

    Ok(())
}

fn verify_preserved_accounts(
    bloom_filter: &[u8],
    reader: PreAccountCellDataReader,
) -> Result<(), Error> {
    debug!("Verify if account is preserved.");

    let account = reader.account().as_readable();
    // debug!("account :{:?}", account);
    // debug!("filter :{:?}", bloom_filter.get(..10));
    let bf = BloomFilter::new_with_data(BLOOM_FILTER_M, BLOOM_FILTER_K, bloom_filter);
    if bf.contains(account.as_slice()) {
        debug!("Account {:?} is reserved.", account);
        return Err(Error::AccountIsReserved);
    }

    Ok(())
}
