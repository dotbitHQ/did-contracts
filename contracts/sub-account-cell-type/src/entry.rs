use alloc::{borrow::ToOwned, vec, vec::Vec};
use ckb_std::{ckb_constants::Source, high_level};
use core::result::Result;
use das_core::{
    assert,
    constants::*,
    debug,
    error::Error,
    sub_account_witness_parser::{SubAccountEditValue, SubAccountWitness, SubAccountWitnessesParser},
    util, verifiers,
    witness_parser::WitnessesParser,
};
use das_types::{
    constants::{AccountStatus, DataType},
    packed::*,
    prelude::{Builder, Entity},
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running sub-account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );

    match action {
        b"enable_sub_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"create_sub_account" | b"edit_sub_account" | b"renew_sub_account" | b"recycle_sub_account" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;
            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verify_transaction_fee_spent_correctly(
                action,
                config_sub_account,
                input_sub_account_cells[0],
                output_sub_account_cells[0],
            )?;

            let mut parent_account = Vec::new();
            match action {
                b"create_sub_account" => {
                    let (input_account_cells, output_account_cells) =
                        util::find_cells_by_type_id_in_inputs_and_outputs(
                            ScriptType::Type,
                            config_main.type_id_table().account_cell(),
                        )?;
                    let input_account_cell_witness =
                        util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
                    let input_account_cell_reader = input_account_cell_witness.as_reader();
                    let output_account_cell_witness =
                        util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
                    let output_account_cell_reader = output_account_cell_witness.as_reader();

                    // TODO Verify AccountCells in inputs and outputs are consistent.

                    parent_account = output_account_cell_reader.account().as_readable();
                    parent_account.extend(ACCOUNT_SUFFIX.as_bytes());
                }
                b"edit_sub_account" => {
                    // Nothing to do here
                }
                b"renew_sub_account" => todo!(),
                b"recycle_sub_account" => todo!(),
                _ => unreachable!(),
            }

            debug!("Start iterating sub-account witnesses ...");

            let mut first_root = &vec![];
            let mut last_root = &vec![];
            let sub_account_parser = SubAccountWitnessesParser::new()?;
            for witness_ret in sub_account_parser.iter() {
                match witness_ret {
                    Ok(witness) => {
                        // Store the first SMT root in the transaction, and verify it later.
                        if first_root.is_empty() {
                            first_root = &witness.prev_root;
                        }

                        debug!(
                            "Verify if the root of witnesses[{}] and witnesses[{}] is sequential.",
                            witness.index,
                            witness.index + 1
                        );

                        match sub_account_parser.get(witness.index + 1) {
                            Some(Ok(next_witness)) => {
                                let current_root = &witness.current_root;
                                let prev_root_of_next = &next_witness.prev_root;

                                assert!(
                                    current_root == prev_root_of_next,
                                    Error::SubAccountCellSMTRootError,
                                    "The roots in sub-account witnesses should be sequential, but witnesses[{}] and witnesses[{}] is not.",
                                    witness.index,
                                    next_witness.index
                                );
                            }
                            Some(Err(err)) => return Err(err),
                            None => {
                                // For the last sub-account witness, there will be no next.
                                // Store the last SMT root in the transaction, and verify it later.
                                last_root = &witness.current_root;
                            }
                        }

                        debug!("Verify if the proof of witnesses[{}] is valid.", witness.index);

                        let sub_account_reader = witness.sub_account.as_reader();
                        match action {
                            b"create_sub_account" => {
                                smt_verify_sub_account_is_creatable(witness)?;

                                debug!("Verify if the account is registrable.");

                                let account_chars = sub_account_reader.account();
                                verifiers::account_cell::verify_account_chars(&parser, account_chars)?;
                                verifiers::account_cell::verify_account_chars_max_length(&parser, account_chars)?;

                                debug!("Verify if every fields of sub-account is filled properly.");

                                verifiers::sub_account_cell::verify_lock(witness.index, sub_account_reader)?;
                                verifiers::sub_account_cell::verify_id(witness.index, sub_account_reader)?;
                                verifiers::sub_account_cell::verify_suffix(
                                    witness.index,
                                    sub_account_reader,
                                    &parent_account,
                                )?;
                                verifiers::sub_account_cell::verify_registered_at(
                                    witness.index,
                                    sub_account_reader,
                                    timestamp,
                                )?;
                                verifiers::sub_account_cell::verify_expired_at(
                                    witness.index,
                                    sub_account_reader,
                                    timestamp,
                                )?;
                                verifiers::sub_account_cell::verify_record_empty(witness.index, sub_account_reader)?;
                                verifiers::sub_account_cell::verify_status(
                                    witness.index,
                                    sub_account_reader,
                                    AccountStatus::Normal,
                                )?;

                                // TODO Verify nonce, enable_sub_account, renew_sub_account_price is 0.

                                // TODO Verify profit is distribute to IncomeCell correctly.
                            }
                            b"edit_sub_account" => {
                                let new_sub_account = generate_new_sub_account_by_edit_value(
                                    witness.sub_account.clone(),
                                    &witness.edit_value,
                                );
                                let new_sub_account_reader = new_sub_account.as_reader();

                                smt_verify_sub_account_is_editable(witness, new_sub_account_reader)?;

                                verifiers::sub_account_cell::verify_expiration(
                                    config_account,
                                    witness.index,
                                    sub_account_reader,
                                    timestamp,
                                )?;
                                verifiers::sub_account_cell::verify_status(
                                    witness.index,
                                    sub_account_reader,
                                    AccountStatus::Normal,
                                )?;

                                // match sub_account_reader.
                            }
                            b"renew_sub_account" => todo!(),
                            b"recycle_sub_account" => todo!(),
                            _ => unreachable!(),
                        }
                    }
                    Err(err) => return Err(err),
                }
            }

            verify_sub_account_cell_is_consistent(input_sub_account_cells[0], output_sub_account_cells[0])?;
            verify_sub_account_cell_smt_root(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                first_root,
                last_root,
            )?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}

fn verify_transaction_fee_spent_correctly(
    action: &[u8],
    config: ConfigCellSubAccountReader,
    input_sub_account: usize,
    output_sub_account: usize,
) -> Result<(), Error> {
    debug!("Check if the fee in the AccountCell is spent correctly.");

    let storage_capacity = u64::from(config.basic_capacity());
    let fee = match action {
        b"create_sub_account" => u64::from(config.create_fee()),
        b"edit_sub_account" => u64::from(config.edit_fee()),
        b"renew_sub_account" => u64::from(config.renew_fee()),
        b"recycle_sub_account" => u64::from(config.recycle_fee()),
        _ => u64::from(config.common_fee()),
    };

    verifiers::common::verify_tx_fee_spent_correctly(
        "AccountCell",
        input_sub_account,
        output_sub_account,
        fee,
        storage_capacity,
    )?;

    Ok(())
}

fn verify_sub_account_cell_is_consistent(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
) -> Result<(), Error> {
    debug!("Verify if the SubAccountCell is consistent in inputs and outputs.");

    let input_sub_account_cell_lock = high_level::load_cell_lock(input_sub_account_cell, Source::Input)?;
    let output_sub_account_cell_lock = high_level::load_cell_lock(output_sub_account_cell, Source::Output)?;

    assert!(
        util::is_entity_eq(&input_sub_account_cell_lock, &output_sub_account_cell_lock),
        Error::SubAccountCellConsistencyError,
        "The SubAccountCell.lock should be consistent in inputs and outputs."
    );

    let input_sub_account_cell_type =
        high_level::load_cell_type(input_sub_account_cell, Source::Input)?.expect("The type script should exist.");
    let output_sub_account_cell_type =
        high_level::load_cell_type(output_sub_account_cell, Source::Output)?.expect("The type script should exist.");

    assert!(
        util::is_entity_eq(&input_sub_account_cell_type, &output_sub_account_cell_type),
        Error::SubAccountCellConsistencyError,
        "The SubAccountCell.type should be consistent in inputs and outputs."
    );

    Ok(())
}

fn verify_sub_account_cell_smt_root(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
    first_root_in_witnesses: &[u8],
    last_root_in_witnesses: &[u8],
) -> Result<(), Error> {
    debug!("Verify if the first SMT root in sub-account witnesses is equal to the SubAccountCell.data in inputs.");

    let first_root = high_level::load_cell_data(input_sub_account_cell, Source::Input)?;

    assert!(
        &first_root == first_root_in_witnesses,
        Error::SubAccountWitnessSMTRootError,
        "The first SMT root in sub-account witnesses should be equal to the SubAccountCell.data in inputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(&first_root),
        util::hex_string(first_root_in_witnesses)
    );

    let last_root = high_level::load_cell_data(output_sub_account_cell, Source::Output)?;

    assert!(
        &last_root == last_root_in_witnesses,
        Error::SubAccountWitnessSMTRootError,
        "The last SMT root in sub-account witnesses should be equal to the SubAccountCell.data in outputs.(root_in_cell: 0x{}, root_in_witness: 0x{})",
        util::hex_string(&last_root),
        util::hex_string(last_root_in_witnesses)
    );

    Ok(())
}

fn smt_verify_sub_account_is_creatable(witness: &SubAccountWitness) -> Result<(), Error> {
    debug!("Verify if the sub-account was not exist in the SMT before.");

    debug!("Verify if the sub-account is in the SMT now.");

    Ok(())
}

fn smt_verify_sub_account_is_editable(
    witness: &SubAccountWitness,
    new_sub_account: SubAccountReader,
) -> Result<(), Error> {
    debug!("Verify if the current state of the sub-account was in the SMT before.");

    debug!("Verify if the new state of the sub-account is in the SMT now.");

    Ok(())
}

fn generate_new_sub_account_by_edit_value(sub_account: SubAccount, edit_value: &SubAccountEditValue) -> SubAccount {
    let sub_account_builder = match edit_value {
        SubAccountEditValue::ExpiredAt(val) => {
            let sub_account_builder = sub_account.as_builder();
            sub_account_builder.expired_at(val.to_owned())
        }
        SubAccountEditValue::Owner(val) | SubAccountEditValue::Manager(val) => {
            let mut lock_builder = sub_account.lock().as_builder();
            let sub_account_builder = sub_account.as_builder();

            lock_builder = lock_builder.args(Bytes::from(val.to_owned()));
            sub_account_builder.lock(lock_builder.build())
        }
        SubAccountEditValue::Records(val) => {
            let sub_account_builder = sub_account.as_builder();
            sub_account_builder.records(val.to_owned())
        }
        _ => unreachable!(),
    };

    sub_account_builder.build()
}
