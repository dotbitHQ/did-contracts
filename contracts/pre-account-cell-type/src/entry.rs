use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed as ckb_packed,
    debug,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock, load_script},
};
use core::result::Result;
use das_core::{
    constants::{ScriptType, ACCOUNT_CELL_BASE_CAPACITY},
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use das_types::{packed::*, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running pre-account-cell-type ======");

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let action = WitnessesParser::find_action(&witnesses)?;

    if action.as_reader().raw_data() == "confirm_proposal".as_bytes() {
        // Move all logic to proposal-cell-type to save cycles, this will save a huge cycles.
        Ok(())
    } else {
        // Parsing all DAS witnesses.
        let parser = WitnessesParser::new(witnesses)?;

        let timestamp = util::load_timestamp()?;
        let config = util::load_config(&parser)?;

        debug!("Find out PreAccountCell ...");

        // Find out PreAccountCells in current transaction.
        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        // Routing by ActionData in witness.
        let action = parser.action.as_reader().raw_data();
        if action == "pre_register".as_bytes() {
            debug!("Route to pre_register action ...");

            debug!(
                "depends on apply-register-cell-type: {}",
                config.as_reader().type_id_table().apply_register_cell()
            );

            // Consuming PreAccountCell is not allowed in pre_register action.
            if old_cells.len() != 0 {
                return Err(Error::PreRegisterFoundInvalidTransaction);
            }
            // Only one PreAccountCell can be created at one time.
            if new_cells.len() != 1 {
                return Err(Error::PreRegisterFoundInvalidTransaction);
            }

            debug!("Find out ApplyRegisterCell ...");

            // Find out ApplyRegisterCells in current transaction.
            let apply_register_cells = util::find_cells_by_type_id(
                ScriptType::Type,
                config.as_reader().type_id_table().apply_register_cell(),
                Source::Input,
            )?;
            // There must be a PreAccountCell created in the transaction.
            if apply_register_cells.len() != 1 {
                return Err(Error::PreRegisterFoundInvalidTransaction);
            }

            debug!("Read data of ApplyRegisterCell ...");

            // Read the hash from outputs_data of the ApplyRegisterCell.
            let index = &apply_register_cells[0];
            let data =
                load_cell_data(index.to_owned(), Source::Input).map_err(|e| Error::from(e))?;
            let apply_register_hash = match data.get(..32) {
                Some(bytes) => bytes,
                _ => return Err(Error::InvalidCellData),
            };
            let apply_register_lock =
                load_cell_lock(index.to_owned(), Source::Input).map_err(|e| Error::from(e))?;

            debug!("Read witness of PreAccountCell ...");

            // Read outputs_data and witness of the PreAccountCell.
            let index = &new_cells[0];
            let data =
                load_cell_data(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;
            let account_id = match data.get(32..) {
                Some(bytes) => Bytes::from(bytes),
                _ => return Err(Error::InvalidCellData),
            };
            let capacity =
                load_cell_capacity(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;

            let (_, _, entity) = util::get_cell_witness(&parser, index.to_owned(), Source::Output)?;
            let pre_account_cell_data =
                PreAccountCellData::from_slice(entity.as_reader().raw_data())
                    .map_err(|_| Error::WitnessEntityDecodingError)?;
            let pre_account_cell_data_reader = pre_account_cell_data.as_reader();

            verify_account_id(&pre_account_cell_data_reader, account_id.as_reader())?;
            verify_apply_register_hash(
                &pre_account_cell_data_reader,
                apply_register_lock.as_reader(),
                apply_register_hash,
            )?;
            verify_create_at_is_correct(&pre_account_cell_data_reader, timestamp)?;
            verify_account_length_is_correct(&pre_account_cell_data_reader)?;
            // Verify quote cell's lock script
            // Verify quote cell exist and has the same value as pre-account-cell.quote field.
            // TODO
            verify_payed_capacity_is_enough(&pre_account_cell_data_reader, capacity)?;
            verify_account_is_available_for_registration(&pre_account_cell_data_reader, timestamp)?;
            verify_account_chars_is_available(&pre_account_cell_data_reader, config.as_reader())?;

        // Verify if the account is preserved.
        // TODO
        } else {
            return Err(Error::ActionNotSupported);
        }

        Ok(())
    }
}

fn verify_account_id(
    reader: &PreAccountCellDataReader,
    account_id: BytesReader,
) -> Result<(), Error> {
    let data_to_hash: Vec<u8> =
        [reader.account().as_readable(), ".bit".as_bytes().to_vec()].concat();
    let hash = util::blake2b_256(data_to_hash.as_slice());

    debug!(
        "Verify account ID in PreAccountCell: {:?} != {:?} {}",
        &hash[..20],
        account_id.raw_data(),
        &hash[..20] != account_id.raw_data()
    );

    if &hash[..20] != account_id.raw_data() {
        return Err(Error::PreRegisterApplyHashIsInvalid);
    }

    Ok(())
}

fn verify_apply_register_hash(
    reader: &PreAccountCellDataReader,
    apply_register_lock_reader: ckb_packed::ScriptReader,
    apply_register_hash: &[u8],
) -> Result<(), Error> {
    let data_to_hash: Vec<u8> = [
        apply_register_lock_reader.args().raw_data().to_vec(),
        reader.account().as_readable(),
        ".bit".as_bytes().to_vec(),
    ]
    .concat();
    let verify_hash = util::blake2b_256(data_to_hash.as_slice());

    debug!(
        "Verify hash in ApplyRegisterCell: {:?} != {:?} {}",
        apply_register_hash,
        verify_hash,
        apply_register_hash != verify_hash
    );

    if apply_register_hash != verify_hash {
        return Err(Error::PreRegisterApplyHashIsInvalid);
    }

    Ok(())
}

fn verify_create_at_is_correct(
    reader: &PreAccountCellDataReader,
    current_timestamp: u64,
) -> Result<(), Error> {
    let create_at: Timestamp = reader.created_at().to_entity();
    if u64::from(create_at) != current_timestamp {
        return Err(Error::PreRegisterApplyHashIsInvalid);
    }

    Ok(())
}

fn verify_account_length_is_correct(reader: &PreAccountCellDataReader) -> Result<(), Error> {
    let price_length: usize = u8::from(reader.price().length()).into();
    if reader.account().len() != price_length {
        return Err(Error::PreRegisterAccountLengthMissMatch);
    }

    Ok(())
}

fn verify_payed_capacity_is_enough(
    reader: &PreAccountCellDataReader,
    capacity: u64,
) -> Result<(), Error> {
    let new_account_price = u64::from(reader.price().new()); // x USD
    let quote = u64::from(reader.quote()); // y USD/CKB
                                           // Register price for 1 year in CKB = x ÷ y
    let register_capacity = new_account_price / quote;
    // Storage price in CKB = AccountCell base capacity + account.length
    let storage_capacity = ACCOUNT_CELL_BASE_CAPACITY - reader.account().len() as u64 + 4;

    debug!("Verify is user payed enough capacity.");
    debug!("CKB required by register: {}", register_capacity);
    debug!("CKB required by storage: {}", storage_capacity);

    if capacity <= register_capacity + storage_capacity {
        return Err(Error::PreRegisterCKBInsufficient);
    }

    Ok(())
}

fn verify_account_is_available_for_registration(
    reader: &PreAccountCellDataReader,
    current_timestamp: u64,
) -> Result<(), Error> {
    let account = reader.account();
    let current = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(current_timestamp as i64, 0),
        Utc,
    );

    debug!(
        "Verify account is currently available for registration. Current datetime: {:#?}",
        current
    );

    let year_2 = Utc.ymd(2022, 1, 1).and_hms(0, 0, 0);
    let year_3 = Utc.ymd(2022, 1, 1).and_hms(0, 0, 0);
    let year_4 = Utc.ymd(2022, 1, 1).and_hms(0, 0, 0);
    if current < year_2 {
        if account.len() <= 7 {
            return Err(Error::PreRegisterAccountCanNotRegisterNow);
        }
    } else if current < year_3 {
        if account.len() <= 6 {
            return Err(Error::PreRegisterAccountCanNotRegisterNow);
        }
    } else if current < year_4 {
        if account.len() <= 5 {
            return Err(Error::PreRegisterAccountCanNotRegisterNow);
        }
    }

    Ok(())
}

fn verify_account_chars_is_available(
    reader: &PreAccountCellDataReader,
    config: ConfigCellDataReader,
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
