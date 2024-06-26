use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use core::str::FromStr;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::*;
use ckb_std::high_level;
use das_core::config::Config;
use das_core::constants::*;
use das_core::error::*;
use das_core::{assert as das_assert, code_to_error, das_assert_custom, data_parser, debug, util, verifiers, warn};
use das_map::map::Map;
use das_map::util as map_util;
use das_types::constants::*;
use das_types::mixer::*;
use das_types::packed::*;
use witness_parser::WitnessesParserV1;

use crate::approval;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running account-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser
        .init()
        .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

    if parser.action != Action::InitAccountChain {
        util::is_system_off()?;
    }

    debug!("Route to {:?} action ...", parser.action.to_string());
    match parser.action {
        Action::InitAccountChain => {
            unreachable!();
        }
        Action::TransferAccount | Action::EditManager | Action::EditRecords | Action::LockAccountForCrossChain => {
            verifiers::account_cell::verify_unlock_role(parser.action, parser.action_params.get_role())?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("AccountCell", &input_account_cells, 1, &output_account_cells, 1)?;

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            verifiers::misc::verify_no_more_cells_with_same_lock(
                sender_lock.as_reader(),
                &input_account_cells,
                Source::Input,
            )?;

            let input_cell_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            let config_account = Config::get_instance().account()?;

            verify_transaction_fee_spent_correctly(
                parser.action,
                config_account,
                input_account_cells[0],
                output_account_cells[0],
            )?;
            if parser.action != Action::LockAccountForCrossChain {
                verify_action_throttle(
                    parser.action,
                    config_account,
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                    timestamp,
                )?;
            }

            verifiers::account_cell::verify_account_expiration(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
            )?;

            match parser.action {
                Action::TransferAccount => action_transfer_account(
                    &input_account_cells,
                    &output_account_cells,
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                )?,
                Action::EditManager => action_edit_manager(
                    &input_account_cells,
                    &output_account_cells,
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                )?,
                Action::EditRecords => action_edit_records(
                    &input_account_cells,
                    &output_account_cells,
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                )?,
                Action::LockAccountForCrossChain => action_lock_account_for_cross_chain(
                    &input_account_cells,
                    &output_account_cells,
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                    timestamp,
                )?,
                _ => unreachable!(),
            }
            //WARNING: migrate it to das-lock
            //util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        Action::RenewAccount => {
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let prices = Config::get_instance().price()?.prices();
            let config_main = Config::get_instance().main()?;
            let config_account = Config::get_instance().account()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verifiers::common::verify_cell_number("AccountCell", &input_account_cells, 1, &output_account_cells, 1)?;

            let input_cell_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec!["expired_at"],
                vec![],
            )?;

            debug!("Verify if the AccountCell is locked for cross chain.");

            let status = u8::from(input_cell_witness_reader.status());
            das_assert!(
                status != (AccountStatus::LockedForCrossChain as u8),
                AccountCellErrorCode::AccountCellStatusLocked,
                "inputs[{}] The AccountCell has been locked for cross chain, it is required to unlock first for renew.",
                input_account_cells[0]
            );

            debug!("Verify if the AccountCell has been expired.");

            let ret = verifiers::account_cell::verify_account_expiration(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
            );
            if let Err(err) = ret {
                das_assert!(
                    err.as_i8() == AccountCellErrorCode::AccountCellInExpirationGracePeriod as i8,
                    AccountCellErrorCode::AccountCellHasExpired,
                    "The AccountCell has been expired."
                );
            } else {
                // Ok
            }

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
            let all_cells = [input_account_cells.clone(), balance_cells.clone()].concat();
            verifiers::misc::verify_no_more_cells_with_same_lock(sender_lock.as_reader(), &all_cells, Source::Input)?;

            debug!("Verify if the profit is distribute correctly.");
            // TODO Unify the following codes to calculate profit from duration.

            let mut profit_map = Map::new();
            let das_wallet_lock = wallet_lock().clone();

            let (input_income_cells, output_income_cells) = util::find_cells_by_type_id_in_inputs_and_outputs(
                ScriptType::Type,
                config_main.type_id_table().income_cell(),
            )?;

            let mut exist_capacity = 0;
            if input_income_cells.len() == 1 {
                let input_income_cell_witness = util::parse_income_cell_witness(input_income_cells[0], Source::Input)?;
                let input_income_cell_witness_reader = input_income_cell_witness.as_reader();

                for item in input_income_cell_witness_reader.records().iter() {
                    if util::is_reader_eq(item.belong_to(), das_wallet_lock.as_reader()) {
                        exist_capacity += u64::from(item.capacity());
                    }
                }
            }

            let output_income_cell_witness = util::parse_income_cell_witness(output_income_cells[0], Source::Output)?;
            let output_income_cell_witness_reader = output_income_cell_witness.as_reader();
            let mut paid = 0;
            for item in output_income_cell_witness_reader.records().iter() {
                if util::is_reader_eq(item.belong_to(), das_wallet_lock.as_reader()) {
                    paid += u64::from(item.capacity());
                }
            }

            das_assert!(
                paid > exist_capacity,
                ErrorCode::IncomeCellConsolidateConditionNotSatisfied,
                "outputs[{}] There is some record in outputs has less capacity than itself in inputs which is not allowed. (belong_to: {})",
                output_income_cells[0],
                das_wallet_lock
            );

            paid -= exist_capacity;

            map_util::add(&mut profit_map, das_wallet_lock.as_slice().to_vec(), paid);
            verifiers::income_cell::verify_income_cells(profit_map)?;

            debug!("Check if the renewal duration is longer than or equal to one year.");

            let input_data = util::load_cell_data(input_account_cells[0], Source::Input)?;
            let output_data = util::load_cell_data(output_account_cells[0], Source::Output)?;
            let input_expired_at = data_parser::account_cell::get_expired_at(&input_data);
            let output_expired_at = data_parser::account_cell::get_expired_at(&output_data);
            let duration = output_expired_at - input_expired_at;

            das_assert!(
                duration >= DAYS_OF_YEAR * DAY_SEC,
                AccountCellErrorCode::AccountCellRenewDurationMustLongerThanYear,
                "The AccountCell renew should be longer than 1 year. (current: {}, expected: >= 31_536_000)",
                duration
            );

            debug!("Check if the expired_at field has been updated correctly based on the capacity paid by the user.");

            let length_in_price = util::get_length_in_price(output_cell_witness_reader.account().len() as u64);
            // Find out register price in from ConfigCellRegister.
            let price = prices
                .iter()
                .find(|item| u8::from(item.length()) == length_in_price)
                .ok_or(ErrorCode::ItemMissing)?;

            let renew_price_in_usd = u64::from(price.renew()); // x USD
            let quote = util::load_oracle_data(OracleCellType::Quote)?;

            let yearly_capacity = util::calc_yearly_register_fee(renew_price_in_usd, quote, 0)?;
            das_assert!(
                paid >= yearly_capacity,
                AccountCellErrorCode::AccountCellRenewDurationMustLongerThanYear,
                "The paid capacity should be at least 1 year. (current: {}, expected: >= {}",
                paid,
                yearly_capacity
            );

            // Renew price for 1 year in CKB = x ÷ y .
            let expected_duration = util::calc_duration_from_paid(paid, renew_price_in_usd, quote, 0)?;
            // The duration can be floated within the range of one day.
            das_assert!(
                duration >= expected_duration - DAY_SEC && duration <= expected_duration + DAY_SEC,
                AccountCellErrorCode::AccountCellRenewDurationBiggerThanPayed,
                "The duration should be equal to {} +/- {}. (current: duration({}), calculation: (paid({}) / (renew_price({}) / quote({}) * 100_000_000) ) * 86400 * 365)",
                expected_duration,
                DAY_SEC,
                duration,
                paid,
                renew_price_in_usd,
                quote
            );

            debug!("Verify if sender get their change properly.");

            let total_input_capacity = util::load_cells_capacity(&balance_cells, Source::Input)?;

            if total_input_capacity > paid {
                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    total_input_capacity - paid,
                )?;
            }

            // The AccountCell can be used as long as it is not modified.
        }
        Action::ConfirmProposal => {
            util::require_type_script(
                TypeScript::ProposalCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::RecycleExpiredAccount => {
            let config_main = Config::get_instance().main()?;
            let config_account = Config::get_instance().account()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_cells,
                &[0, 1],
                &output_cells,
                &[0],
            )?;

            let input_prev_cell_witness = util::parse_account_cell_witness(input_cells[0], Source::Input)?;
            let input_prev_cell_witness_reader = input_prev_cell_witness.as_reader();
            let output_prev_cell_witness = util::parse_account_cell_witness(output_cells[0], Source::Output)?;
            let output_prev_cell_witness_reader = output_prev_cell_witness.as_reader();

            verifiers::account_cell::verify_account_capacity_not_decrease(input_cells[0], output_cells[0])?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_cells[0],
                output_cells[0],
                &input_prev_cell_witness_reader,
                &output_prev_cell_witness_reader,
                None,
                vec!["next"],
                vec![],
            )?;

            debug!("Verify if the AccountCell has been expired.");

            let ret = verifiers::account_cell::verify_account_expiration(
                config_account,
                input_cells[1],
                Source::Input,
                timestamp,
            );
            if let Err(err) = ret {
                das_assert!(
                    err.as_i8() == AccountCellErrorCode::AccountCellHasExpired as i8,
                    AccountCellErrorCode::AccountCellStillCanNotRecycle,
                    "The AccountCell is still disable for recycling."
                );
            } else {
                das_assert!(
                    false,
                    AccountCellErrorCode::AccountCellStillCanNotRecycle,
                    "The AccountCell is still disable for recycling."
                );
            }

            debug!("Verify if the AccountCell is in status which could be recycled.");

            // manual::verify_account_status
            let expired_account_witness = util::parse_account_cell_witness(input_cells[1], Source::Input)?;
            let expired_account_witness_reader = expired_account_witness.as_reader();
            let account_cell_status = u8::from(expired_account_witness_reader.status());

            das_assert!(
                account_cell_status == AccountStatus::Normal as u8
                    || account_cell_status == AccountStatus::LockedForCrossChain as u8,
                AccountCellErrorCode::AccountCellStatusLocked,
                "inputs[{}] The AccountCell.witness.status should be Normal or LockedForCrossChain .",
                input_cells[1]
            );

            debug!("Verify if the SubAccountCell has been recycled either.");

            let mut refund_from_sub_account_cell_to_das = 0;
            let mut refund_from_sub_account_cell_to_owner = 0;

            // TODO find a better way to handle multiple version of witness
            let enable_sub_account = match expired_account_witness_reader.version() {
                3 => {
                    let reader = expired_account_witness_reader.try_into_v3().unwrap();
                    u8::from(reader.enable_sub_account())
                }
                4 => {
                    let reader = expired_account_witness_reader.try_into_latest().unwrap();
                    u8::from(reader.enable_sub_account())
                }
                _ => SubAccountEnableStatus::Off as u8,
            };

            if enable_sub_account == SubAccountEnableStatus::On as u8 {
                debug!("Verify if the SubAccountCell is recycled properly.");

                let sub_account_type_id = config_main.type_id_table().sub_account_cell();
                let (input_sub_account_cells, output_sub_account_cells) =
                    util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, sub_account_type_id)?;

                verifiers::common::verify_cell_number_and_position(
                    "SubAccountCell",
                    &input_sub_account_cells,
                    &[2],
                    &output_sub_account_cells,
                    &[],
                )?;

                verifiers::sub_account_cell::verify_sub_account_parent_id(
                    input_sub_account_cells[0],
                    Source::Input,
                    expired_account_witness_reader.id().raw_data(),
                )?;

                let total_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
                let sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
                refund_from_sub_account_cell_to_das =
                    data_parser::sub_account_cell::get_das_profit(&sub_account_data).unwrap();
                refund_from_sub_account_cell_to_owner = total_capacity - refund_from_sub_account_cell_to_das;
            }

            debug!("Verify if the AccountCell is recycled properly.");

            // manual::verify_account_contiguous
            let prev_account_input_data = high_level::load_cell_data(input_cells[0], Source::Input)?;
            let expired_account_data = high_level::load_cell_data(input_cells[1], Source::Input)?;
            let prev_account_input_next = data_parser::account_cell::get_next(&prev_account_input_data);
            let expired_account_id = data_parser::account_cell::get_id(&expired_account_data);

            das_assert!(
                prev_account_input_next == expired_account_id,
                AccountCellErrorCode::AccountCellMissingPrevAccount,
                "inputs[{}] The AccountCell.next should be 0x{} .",
                input_cells[0],
                util::hex_string(expired_account_id)
            );

            // manual::verify_account_next_updated
            let prev_account_output_data = high_level::load_cell_data(output_cells[0], Source::Output)?;
            let prev_account_output_next = data_parser::account_cell::get_next(&prev_account_output_data);
            let expired_account_next = data_parser::account_cell::get_next(&expired_account_data);

            das_assert!(
                prev_account_output_next == expired_account_next,
                AccountCellErrorCode::AccountCellNextUpdateError,
                "outputs[{}] The AccountCell.next should be updated to 0x{} .",
                output_cells[0],
                util::hex_string(expired_account_next)
            );

            debug!("Verify if all the refunds has been refund properly.");

            let expired_account_capacity = high_level::load_cell_capacity(input_cells[1], Source::Input)?;
            let available_fee = u64::from(config_account.common_fee());
            let refund_lock = util::derive_owner_lock_from_cell(input_cells[1], Source::Input)?;
            let refund_args = refund_lock.as_reader().args().raw_data();
            let owner_args = data_parser::das_lock_args::get_owner_lock_args(refund_args);

            if owner_args != &CROSS_CHAIN_BLACK_ARGS {
                // If the lock is not the black hole lock, then the refund should be refunded to current owner.

                debug!("The lock is not the black hole lock, so refund normally.");

                verifiers::misc::verify_user_get_change(
                    config_main,
                    refund_lock.as_reader(),
                    expired_account_capacity + refund_from_sub_account_cell_to_owner - available_fee,
                )?;

                if refund_from_sub_account_cell_to_das >= CELL_BASIC_CAPACITY {
                    verifiers::common::verify_das_get_change(refund_from_sub_account_cell_to_das)?;
                } else {
                    debug!(
                        "The profit of DAS is {} shannon, so no need to refund to DAS.",
                        refund_from_sub_account_cell_to_das
                    );
                }
            } else {
                // If the lock is the black hole lock, then all the refunds should be sent to DAS first.

                debug!("The lock is the black hole lock, so all the refunds should be sent to DAS first.");

                let das_wallet_lock = wallet_lock();
                let das_wallet_cells =
                    util::find_cells_by_script(ScriptType::Lock, das_wallet_lock.as_reader().into(), Source::Output)?;
                let expected_das_wallet_cells_count = if refund_from_sub_account_cell_to_das >= CELL_BASIC_CAPACITY {
                    // If the profit of DAS is more than a cell's basic capacity, there should be a single cell carrying the profit.
                    2
                } else {
                    // Else, the keeper could deal with the refund casually.
                    1
                };

                verifiers::common::verify_cell_number(
                    "DASWallet",
                    &[],
                    0,
                    &das_wallet_cells,
                    expected_das_wallet_cells_count,
                )?;

                for i in das_wallet_cells.iter() {
                    let type_hash = high_level::load_cell_type_hash(*i, Source::Output)?;
                    das_assert!(
                        type_hash.is_none(),
                        ErrorCode::InvalidTransactionStructure,
                        "outputs[{}] The cells to DAS should not contains any type script.",
                        i
                    );
                }

                // The refund to owner should be always more than a cell's basic capacity because it contains the capacity of the SubAccountCell.
                let capacity = high_level::load_cell_capacity(das_wallet_cells[0], Source::Output)?;
                das_assert!(
                    capacity == expired_account_capacity + refund_from_sub_account_cell_to_owner - available_fee,
                    ErrorCode::ChangeError,
                    "outputs[{}] The ChangeCell to DAS should be {} shannon, but {} found.",
                    das_wallet_cells[0],
                    expired_account_capacity + refund_from_sub_account_cell_to_owner - available_fee,
                    capacity
                );

                if expected_das_wallet_cells_count == 2 {
                    let capacity = high_level::load_cell_capacity(das_wallet_cells[1], Source::Output)?;
                    das_assert!(
                        capacity == refund_from_sub_account_cell_to_das,
                        ErrorCode::ChangeError,
                        "outputs[{}] The ChangeCell to DAS should be {} shannon, but {} found.",
                        das_wallet_cells[1],
                        refund_from_sub_account_cell_to_das,
                        capacity
                    );
                }
            }
        }
        Action::StartAccountSale => {
            util::require_type_script(
                TypeScript::AccountSaleCellType,
                Source::Output,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::CancelAccountSale | Action::BuyAccount => {
            util::require_type_script(
                TypeScript::AccountSaleCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::AcceptOffer => {
            util::require_type_script(
                TypeScript::OfferCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::ForceRecoverAccountStatus => {
            let config_main = Config::get_instance().main()?;
            let config_account = Config::get_instance().account()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position("AccountCell", &input_cells, &[0], &output_cells, &[0])?;

            let input_cell_witness = util::parse_account_cell_witness(input_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(output_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            debug!("Verify if the AccountCell is consistent in inputs and outputs.");

            verifiers::account_cell::verify_account_capacity_not_decrease(input_cells[0], output_cells[0])?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_cells[0],
                output_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec![],
                vec!["status"],
            )?;

            debug!("Verify if the AccountCell status updated correctly.");

            let input_status = u8::from(input_cell_witness_reader.status());
            das_assert!(
                input_status != AccountStatus::Normal as u8,
                ErrorCode::InvalidTransactionStructure,
                "The AccountCell in inputs should not be in NORMAL status."
            );

            verifiers::account_cell::verify_status(
                &output_cell_witness_reader,
                AccountStatus::Normal,
                output_cells[0],
                Source::Output,
            )?;

            debug!("Verify if the AccountCell is actually expired.");

            let ret = verifiers::account_cell::verify_account_expiration(
                config_account,
                input_cells[0],
                Source::Input,
                timestamp,
            );
            if let Err(err) = ret {
                das_assert!(
                    err.as_i8() == AccountCellErrorCode::AccountCellInExpirationAuctionPeriod as i8
                        || err.as_i8() == AccountCellErrorCode::AccountCellInExpirationAuctionConfirmationPeriod as i8
                        || err.as_i8() == AccountCellErrorCode::AccountCellHasExpired as i8,
                    AccountCellErrorCode::AccountCellIsNotExpired,
                    "The AccountCell is still not expired."
                );
            } else {
                das_assert!(
                    false,
                    AccountCellErrorCode::AccountCellIsNotExpired,
                    "The AccountCell is still not expired."
                );
            }

            let capacity_should_recycle;
            let cell;
            if input_status == AccountStatus::Selling as u8 {
                let input_cell_data = high_level::load_cell_data(input_cells[0], Source::Input)?;
                let account = data_parser::account_cell::get_account(&input_cell_data);

                let type_id = Config::get_instance().main()?.type_id_table().account_sale_cell();
                let (input_sale_cells, output_sale_cells) =
                    util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id)?;
                verifiers::common::verify_cell_number_and_position(
                    "AccountSaleCell",
                    &input_sale_cells,
                    &[1],
                    &output_sale_cells,
                    &[],
                )?;

                let cell_witness = util::parse_account_sale_cell_witness(input_sale_cells[0], Source::Input)?;
                let cell_witness_reader = cell_witness.as_reader();

                das_assert!(
                    account == cell_witness_reader.account().raw_data(),
                    ErrorCode::AccountSaleCellAccountIdInvalid,
                    "The account in AccountCell and AccountSaleCell should be the same."
                );

                cell = input_sale_cells[0];
            } else {
                // TODO Verify the account in AccountCell and AccountAuctionCell is the same.
                cell = 0;
            }
            capacity_should_recycle = high_level::load_cell_capacity(cell, Source::Input)?;

            debug!(
                "Found the capacity should be recycled is {} shannon.",
                capacity_should_recycle
            );

            let balance_cell_type_id = config_main.type_id_table().balance_cell();
            let (input_balance_cells, outputs_balance_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, balance_cell_type_id)?;
            verifiers::common::verify_cell_number_and_position(
                "BalanceCell",
                &input_balance_cells,
                &[],
                &outputs_balance_cells,
                &[1],
            )?;

            let expected_lock = util::derive_owner_lock_from_cell(input_cells[0], Source::Input)?;
            let current_lock = high_level::load_cell_lock(outputs_balance_cells[0], Source::Output)?.into();
            das_assert!(
                util::is_entity_eq(&expected_lock, &current_lock),
                ErrorCode::AccountSaleCellRefundError,
                "The lock receiving the refund is incorrect.(expected: {}, current: {})",
                expected_lock,
                current_lock
            );

            let expected_capacity = capacity_should_recycle - 20_000;
            let current_capacity = high_level::load_cell_capacity(outputs_balance_cells[0], Source::Output)?;
            das_assert!(
                current_capacity >= expected_capacity,
                ErrorCode::AccountSaleCellRefundError,
                "The capacity refunding is incorrect.(expected: {}, current: {})",
                expected_capacity,
                current_capacity
            );
        }
        Action::EnableSubAccount => {
            // CAREFUL! This action is intentionally ignoring EIP712 verification.
            // verify_eip712_hashes(transfer_account_to_semantic)?;

            verifiers::account_cell::verify_unlock_role(parser.action, parser.action_params.get_role())?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;
            let config_main = Config::get_instance().main()?;
            let config_account = Config::get_instance().account()?;
            let config_sub_account = Config::get_instance().sub_account()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[0],
                &output_account_cells,
                &[0],
            )?;

            let input_account_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
            let input_account_witness_reader = input_account_witness.as_reader();
            let output_account_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
            let output_account_witness_reader = output_account_witness.as_reader();

            debug!("Verify if the AccountCell is locked or expired.");

            verifiers::account_cell::verify_status(
                &input_account_witness_reader,
                AccountStatus::Normal,
                input_account_cells[0],
                Source::Input,
            )?;
            verifiers::account_cell::verify_account_expiration(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
            )?;

            debug!("Verify if every aspects of the AccountCell is consistent.");

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_account_witness_reader,
                &output_account_witness_reader,
                None,
                vec![],
                vec!["enable_sub_account", "renew_sub_account_price"],
            )?;

            debug!("Verify if the AccountCell can enable sub-account function.");

            // TODO find a better way to handle multiple version of witness
            let enable_status = match input_account_witness_reader.version() {
                3 => {
                    let reader = input_account_witness_reader.try_into_v3().unwrap();
                    u8::from(reader.enable_sub_account())
                }
                4 => {
                    let reader = input_account_witness_reader.try_into_latest().unwrap();
                    u8::from(reader.enable_sub_account())
                }
                _ => SubAccountEnableStatus::Off as u8,
            };

            das_assert!(
                enable_status == SubAccountEnableStatus::Off as u8,
                AccountCellErrorCode::AccountCellPermissionDenied,
                "{:?}[{}] Only AccountCells with enable_sub_account field is {} can enable its sub-account function.",
                Source::Input,
                input_account_cells[0],
                SubAccountEnableStatus::Off as u8
            );

            match output_account_witness_reader.try_into_latest() {
                Ok(reader) => {
                    let enable_status = u8::from(reader.enable_sub_account());
                    das_assert!(
                        enable_status == SubAccountEnableStatus::On as u8,
                        AccountCellErrorCode::AccountCellPermissionDenied,
                        "{:?}[{}]The AccountCell.enable_sub_account should be {} .",
                        Source::Output,
                        output_account_cells[0],
                        SubAccountEnableStatus::On as u8
                    );
                }
                Err(_) => {
                    warn!(
                        "{:?}[{}] The version of this AccountCell should be latest.",
                        Source::Output,
                        output_account_cells[0]
                    );
                    return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
                }
            }

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            let balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;
            let all_cells = [input_account_cells.clone(), balance_cells.clone()].concat();
            verifiers::misc::verify_no_more_cells_with_same_lock(sender_lock.as_reader(), &all_cells, Source::Input)?;

            debug!("Verify if the SubAccountCell is created properly.");

            let sub_account_cell_type_id = config_main.type_id_table().sub_account_cell();
            let (input_sub_account_cells, output_sub_account_cells) =
                util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, sub_account_cell_type_id)?;
            // manual::verify_sub_account_cell_created
            verifiers::common::verify_cell_number_and_position(
                "SubAccountCell",
                &input_sub_account_cells,
                &[],
                &output_sub_account_cells,
                &[1],
            )?;

            verifiers::misc::verify_always_success_lock(output_sub_account_cells[0], Source::Output)?;

            let sub_account_cell_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let expected_capacity =
                u64::from(config_sub_account.basic_capacity()) + u64::from(config_sub_account.prepared_fee_capacity());

            das_assert!(
                sub_account_cell_capacity == expected_capacity,
                SubAccountCellErrorCode::SubAccountCellCapacityError,
                "The initial capacity of SubAccountCell should be equal to ConfigCellSubAccount.basic_capacity + ConfigCellSubAccount.prepared_fee_capacity .(expected: {}, current: {})",
                expected_capacity,
                sub_account_cell_capacity
            );

            let type_script = high_level::load_cell_type(output_sub_account_cells[0], Source::Output)?.unwrap();
            let account_id = type_script.as_reader().args().raw_data();
            let expected_account_id = output_account_witness_reader.id().raw_data();

            das_assert!(
                account_id == expected_account_id,
                SubAccountCellErrorCode::SubAccountCellAccountIdError,
                "The type.args of SubAccountCell should be the same with the AccountCell.witness.id .(expected: {}, current: {})",
                util::hex_string(expected_account_id),
                util::hex_string(account_id)
            );

            let sub_account_outputs_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;
            verifiers::sub_account_cell::verify_cell_initial_properties(&sub_account_outputs_data)?;

            debug!("Verify if sender get their change properly.");

            let total_input_capacity = util::load_cells_capacity(&balance_cells, Source::Input)?;
            let available_fee = u64::from(config_account.common_fee());
            if total_input_capacity > sub_account_cell_capacity {
                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    total_input_capacity - sub_account_cell_capacity - available_fee,
                )?;
            }
        }
        Action::ConfigSubAccount => {
            util::require_type_script(
                TypeScript::SubAccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::UnlockAccountForCrossChain => {
            debug!("Verify if there is no redundant AccountCells.");

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[0],
                &output_account_cells,
                &[0],
            )?;

            let input_cell_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            let config_account = Config::get_instance().account()?;

            // include: common::verify_tx_fee_spent_correctly
            verify_transaction_fee_spent_correctly(
                parser.action,
                config_account,
                input_account_cells[0],
                output_account_cells[0],
            )?;

            verifiers::account_cell::verify_status(
                &input_cell_witness_reader,
                AccountStatus::LockedForCrossChain,
                input_account_cells[0],
                Source::Input,
            )?;

            verifiers::account_cell::verify_account_data_consistent(
                input_account_cells[0],
                output_account_cells[0],
                vec![],
            )?;

            debug!("Verify if the lock.args is changed during the unlock transaction.");

            // CAREFUL! The owner lock may be changed or not changed, only the keepers know it, so we skip verification here.
            let input_lock =
                high_level::load_cell_lock(input_account_cells[0], Source::Input).map_err(Error::<ErrorCode>::from)?;
            let input_args = input_lock.as_reader().args().raw_data();
            let output_lock = high_level::load_cell_lock(output_account_cells[0], Source::Output)
                .map_err(Error::<ErrorCode>::from)?;
            let output_args = output_lock.as_reader().args().raw_data();
            let (owner_changed, _) = util::diff_das_lock_args(input_args, output_args);

            if owner_changed {
                // The lock is changed, so the records must be cleared.
                verifiers::account_cell::verify_account_witness_consistent(
                    input_account_cells[0],
                    output_account_cells[0],
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                    vec!["status", "records"],
                )?;
                verifiers::account_cell::verify_account_witness_record_empty(
                    &output_cell_witness_reader,
                    output_account_cells[0],
                    Source::Output,
                )?;
            } else {
                // The lock is not changed, so the records must be kept.
                verifiers::account_cell::verify_account_witness_consistent(
                    input_account_cells[0],
                    output_account_cells[0],
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                    vec!["status"],
                )?;
            }

            das_assert!(
                util::is_das_lock_owner_manager_same(output_args),
                ErrorCode::CrossChainUnlockError,
                "The owner lock is not the same with the manager lock in outputs."
            );

            verify_account_is_unlocked_for_cross_chain(output_account_cells[0], &output_cell_witness_reader)?;

            //verify_multi_sign(input_account_cells[0], config_main.das_lock_type_id_table())?;
        }
        Action::BidExpiredAccountDutchAuction => {
            //get configs
            let config_main = Config::get_instance().main()?;
            let config_account = Config::get_instance().account()?;
            let config_prices = Config::get_instance().price()?.prices();

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;
            let quote = util::load_oracle_data(OracleCellType::Quote)?;

            debug!("Verify if there is no redundant AccountCells.");
            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[0],
                &output_account_cells,
                &[0],
            )?;

            //There can only be account cell and dp cell in inputs
            verifiers::account_cell::verify_account_no_other_type_cell_use_das_lock_in_inputs(
                config_main.type_id_table(),
            )?;

            //get account witness parser
            let input_cell_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            //transaction fee paid by input AccountCell or did_svr
            verify_transaction_fee_spent_correctly(
                parser.action,
                config_account,
                input_account_cells[0],
                output_account_cells[0],
            )?;

            verifiers::account_cell::verify_account_data_consistent(
                input_account_cells[0],
                output_account_cells[0],
                vec!["expired_at"],
            )?;
            verifiers::account_cell::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                vec![
                    "registered_at",
                    "last_transfer_account_at",
                    "last_edit_manager_at",
                    "last_edit_records_at",
                    "records",
                    "status",
                ],
            )?;

            // let records_len = output_cell_witness_reader.records().len();
            // das_assert!(
            //     records_len == 1,
            //     ErrorCode::InvalidTransactionStructure,
            //     "The records field in output AccountCell should only one, but {}.",
            //     records_len
            // );

            verifiers::account_cell::verify_status_v2(
                &input_cell_witness_reader,
                &[AccountStatus::Normal, AccountStatus::LockedForCrossChain],
                input_account_cells[0],
                Source::Input,
            )?;
            verifiers::account_cell::verify_status(
                &output_cell_witness_reader,
                AccountStatus::Normal,
                output_account_cells[0],
                Source::Output,
            )?;

            debug!("Check whether the date of the account cell in the output is one year later.");

            let output_data = util::load_cell_data(output_account_cells[0], Source::Output)?;
            let output_expired_at = data_parser::account_cell::get_expired_at(&output_data);
            let output_registered_at = u64::from(output_cell_witness_reader.registered_at());
            let output_last_transfer_account_at = u64::from(output_cell_witness_reader.last_transfer_account_at());
            let output_last_edit_manager_at = u64::from(output_cell_witness_reader.last_edit_manager_at());
            let output_last_edit_records_at = u64::from(output_cell_witness_reader.last_edit_records_at());

            // register_at should be the same as timestamp
            das_assert_custom!(
                output_registered_at == timestamp,
                "The register_at field in output AccountCell should be changed to current time.",
                output_last_transfer_account_at == 0,
                "The last_transfer_account_at in the output AccountCell should be set to 0.",
                output_last_edit_manager_at == 0,
                "The last_edit_manager_at in the output AccountCell should be set to 0.",
                output_last_edit_records_at == 0,
                "The last_edit_records_at in the output AccountCell should be set to 0."
            );

            //expired_at should be timestamp + 1year
            let duration = output_expired_at - timestamp;
            das_assert!(
                duration == YEAR_SEC,
                ErrorCode::InvalidTransactionStructure,
                "The expired_at field in outputs AccountCell should be changed to {}.",
                timestamp + YEAR_SEC
            );

            debug!("Check if the old owner has received the refund.");

            let expired_account_capacity = high_level::load_cell_capacity(input_account_cells[0], Source::Input)?;
            let available_fee = u64::from(config_account.common_fee());
            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            let sender_args = sender_lock.as_reader().args().raw_data();
            let owner_args = data_parser::das_lock_args::get_owner_lock_args(sender_args);

            //If it is a black hole address, the contract does not verify the returned funds.
            if owner_args != &CROSS_CHAIN_BLACK_ARGS {
                debug!("Check if account cell refund to old owner properly.");

                verifiers::misc::verify_user_get_change(
                    config_main,
                    sender_lock.as_reader(),
                    expired_account_capacity - available_fee,
                )?;
            }

            // Get basic capacity
            let account_name_storage = data_parser::account_cell::get_account(&output_data).len() as u64;
            let receiver_lock = util::derive_owner_lock_from_cell(output_account_cells[0], Source::Output)?;
            let storage_capacity = util::calc_account_storage_capacity(
                config_account,
                account_name_storage,
                receiver_lock.args().as_reader().into(),
            );

            debug!("The storage capacity is {} shannon", storage_capacity);

            //warning: there is a possibility of overflow in u64 here.
            let storage_price_in_usd = storage_capacity * quote / ONE_CKB;

            // Calculate the price when bid
            let length_in_price = util::get_length_in_price(output_cell_witness_reader.account().len() as u64);

            // Find out register price in from ConfigCellRegister.
            let price = config_prices
                .iter()
                .find(|item| u8::from(item.length()) == length_in_price)
                .ok_or(ErrorCode::ItemMissing)?;

            let new_price_in_usd = u64::from(price.new()); // x USD

            let basic_price_in_usd = storage_price_in_usd + new_price_in_usd;
            debug!(
                "The basic price is {} USD = {}(storage_price) + {}(new_price)",
                basic_price_in_usd, storage_price_in_usd, new_price_in_usd
            );

            //Check owner and manager is equal.
            let receiver_lock_args = receiver_lock.args();
            let receiver_lock_args_u8 = receiver_lock_args.as_reader().raw_data();
            let receiver_owner = data_parser::das_lock_args::get_owner_lock_args(receiver_lock_args_u8);
            let receiver_manager = data_parser::das_lock_args::get_manager_lock_args(receiver_lock_args_u8);
            das_assert!(
                receiver_owner == receiver_manager,
                AccountCellErrorCode::AccountCellPermissionDenied,
                "The owner and manager of the AccountCell in the outputs should be the same."
            );

            //Get the price paid by the user during the auction.
            let type_id_table_reader = config_main.type_id_table();
            let (input_dp_cells, output_dp_cells) = util::find_cells_by_type_id_in_inputs_and_outputs(
                ScriptType::Type,
                type_id_table_reader.dpoint_cell(),
            )?;
            let bid_price =
                util::get_spent_dpoint_by_lock(receiver_lock.as_reader(), &input_dp_cells, &output_dp_cells)?;

            debug!("The amount spent by the user is {} USD.", bid_price);

            // Verify that this account is within the Dutch auction period.
            debug!("Check that the amount complies with Dutch auction price rules.");
            verifiers::account_cell::verify_account_in_auction(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
                bid_price,
                basic_price_in_usd,
            )?;
            //WARNING: migrate it to das-lock
            //util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        Action::CreateApproval | Action::DelayApproval | Action::RevokeApproval | Action::FulfillApproval => {
            action_approve()?
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}

fn action_transfer_account<'a>(
    input_account_cells: &[usize],
    output_account_cells: &[usize],
    input_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    verifiers::account_cell::verify_status(
        &input_cell_witness_reader,
        AccountStatus::Normal,
        input_account_cells[0],
        Source::Input,
    )?;

    verifiers::account_cell::verify_account_cell_consistent_with_exception(
        input_account_cells[0],
        output_account_cells[0],
        &input_cell_witness_reader,
        &output_cell_witness_reader,
        Some("owner"),
        vec![],
        vec!["last_transfer_account_at", "records"],
    )?;

    verifiers::account_cell::verify_account_witness_record_empty(
        &output_cell_witness_reader,
        output_account_cells[0],
        Source::Output,
    )?;

    Ok(())
}

fn action_edit_manager<'a>(
    input_account_cells: &[usize],
    output_account_cells: &[usize],
    input_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    verifiers::account_cell::verify_status(
        &input_cell_witness_reader,
        AccountStatus::Normal,
        input_account_cells[0],
        Source::Input,
    )?;

    verifiers::account_cell::verify_account_cell_consistent_with_exception(
        input_account_cells[0],
        output_account_cells[0],
        &input_cell_witness_reader,
        &output_cell_witness_reader,
        Some("manager"),
        vec![],
        vec!["last_edit_manager_at"],
    )?;

    Ok(())
}

fn action_edit_records<'a>(
    input_account_cells: &[usize],
    output_account_cells: &[usize],
    input_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    verifiers::account_cell::verify_status_v2(
        &input_cell_witness_reader,
        &[AccountStatus::Normal, AccountStatus::ApprovedTransfer],
        input_account_cells[0],
        Source::Input,
    )?;

    verifiers::account_cell::verify_account_cell_consistent_with_exception(
        input_account_cells[0],
        output_account_cells[0],
        &input_cell_witness_reader,
        &output_cell_witness_reader,
        None,
        vec![],
        vec!["records", "last_edit_records_at"],
    )?;
    verifiers::account_cell::verify_records_keys(output_cell_witness_reader.records())?;

    Ok(())
}

fn action_lock_account_for_cross_chain<'a>(
    input_account_cells: &[usize],
    output_account_cells: &[usize],
    input_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_cell_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    timestamp: u64,
) -> Result<(), Box<dyn ScriptError>> {
    verifiers::account_cell::verify_status(
        &input_cell_witness_reader,
        AccountStatus::Normal,
        input_account_cells[0],
        Source::Input,
    )?;

    verifiers::account_cell::verify_account_cell_consistent_with_exception(
        input_account_cells[0],
        output_account_cells[0],
        &input_cell_witness_reader,
        &output_cell_witness_reader,
        None,
        vec![],
        vec!["status"],
    )?;

    verify_account_is_locked_for_cross_chain(output_account_cells[0], &output_cell_witness_reader, timestamp)?;

    Ok(())
}

fn action_approve() -> Result<(), Box<dyn ScriptError>> {
    let parser = WitnessesParserV1::get_instance();

    verifiers::account_cell::verify_unlock_role(parser.action, parser.action_params.get_role())?;

    let timestamp = util::load_oracle_data(OracleCellType::Time)?;

    let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    verifiers::common::verify_cell_number_and_position(
        "AccountCell",
        &input_account_cells,
        &[0],
        &output_account_cells,
        &[0],
    )?;

    debug!("Verify if there is no redundant cells in inputs.");

    // WARNING! This is required for the revoke_approval and fulfill_approval transaction.
    verifiers::misc::verify_no_more_cells(&input_account_cells, Source::Input)?;

    let input_cell_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
    let input_cell_witness_reader = input_cell_witness.as_reader();
    let output_cell_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
    let output_cell_witness_reader = output_cell_witness.as_reader();

    let config_account = Config::get_instance().account()?;

    verify_transaction_fee_spent_correctly(
        parser.action,
        config_account,
        input_account_cells[0],
        output_account_cells[0],
    )?;

    // TODO The codes above is duplicate with the transfer action.

    match output_cell_witness_reader.try_into_latest() {
        Ok(_reader) => {}
        Err(_) => {
            warn!(
                "{:?}[{}] The version of this AccountCell should be latest.",
                Source::Output,
                output_account_cells[0]
            );
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
    };

    match parser.action {
        Action::CreateApproval => {
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec![],
                vec!["status", "approval"],
            )?;

            let approval_action = get_approval_action(&output_cell_witness_reader)?;

            match approval_action {
                AccountApprovalAction::Transfer => {
                    approval::transfer_approval_create(
                        timestamp,
                        input_account_cells[0],
                        output_account_cells[0],
                        input_cell_witness_reader,
                        output_cell_witness_reader,
                    )?;
                }
            }
        }
        Action::DelayApproval => {
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec![],
                vec!["approval"],
            )?;

            let approval_action = get_approval_action(&output_cell_witness_reader)?;

            match approval_action {
                AccountApprovalAction::Transfer => {
                    approval::transfer_approval_delay(
                        input_account_cells[0],
                        output_account_cells[0],
                        input_cell_witness_reader,
                        output_cell_witness_reader,
                    )?;
                }
            }
        }
        Action::RevokeApproval => {
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                None,
                vec![],
                vec!["status", "approval"],
            )?;

            let approval_action = get_approval_action(&input_cell_witness_reader)?;

            match approval_action {
                AccountApprovalAction::Transfer => {
                    approval::transfer_approval_revoke(
                        timestamp,
                        input_account_cells[0],
                        output_account_cells[0],
                        input_cell_witness_reader,
                        output_cell_witness_reader,
                    )?;
                }
            }
        }
        Action::FulfillApproval => {
            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                Some("owner"),
                vec![],
                vec!["status", "approval", "records"],
            )?;

            let approval_action = get_approval_action(&input_cell_witness_reader)?;

            match approval_action {
                AccountApprovalAction::Transfer => {
                    let sealed_until = approval::transfer_approval_fulfill(
                        input_account_cells[0],
                        output_account_cells[0],
                        input_cell_witness_reader,
                        output_cell_witness_reader,
                    )?;

                    if timestamp > sealed_until {
                        debug!("The approval is already released, so anyone can fulfill it.");
                    } else {
                        debug!("The approval is not released, so its signature should be verified by das-lock.");
                    }
                }
            }
        }
        _ => {
            warn!("Action {} is not a valid approval action.", parser.action.to_string());
            return Err(code_to_error!(ErrorCode::ActionNotSupported));
        }
    }

    Ok(())
}

fn get_approval_action<'a>(
    witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<AccountApprovalAction, Box<dyn ScriptError>> {
    let reader = match witness_reader.try_into_latest() {
        Ok(reader) => reader,
        Err(_) => {
            warn!("Only latest version of AccountCellData should used here.");
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
    };

    let action_bytes = reader.approval().action().raw_data();
    let action_string = String::from_utf8(action_bytes.to_vec())
        .map_err(|_| code_to_error!(AccountCellErrorCode::ApprovalActionUndefined))?;
    let approval_action = AccountApprovalAction::from_str(&action_string)
        .map_err(|_| code_to_error!(AccountCellErrorCode::ApprovalActionUndefined))?;

    Ok(approval_action)
}

fn verify_transaction_fee_spent_correctly(
    action: Action,
    config: ConfigCellAccountReader,
    input_account_index: usize,
    output_account_index: usize,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if the fee in the AccountCell is spent correctly.");

    // TODO MIXIN Fix this with new data structure.
    let lock = high_level::load_cell_lock(input_account_index, Source::Input)?;
    let lock_type = data_parser::das_lock_args::get_owner_type(lock.as_reader().args().raw_data());
    let basic_capacity = if lock_type == DasLockType::MIXIN as u8 {
        23_000_000_000u64
    } else {
        u64::from(config.basic_capacity())
    };

    let input_data = util::load_cell_data(input_account_index, Source::Input)?;
    let account_length = data_parser::account_cell::get_account(&input_data).len() as u64;

    let fee = match action {
        Action::TransferAccount => u64::from(config.transfer_account_fee()),
        Action::EditManager => u64::from(config.edit_manager_fee()),
        Action::EditRecords => u64::from(config.edit_records_fee()),
        _ => u64::from(config.common_fee()),
    };
    let storage_capacity = basic_capacity + account_length * ONE_CKB;

    verifiers::common::verify_tx_fee_spent_correctly(
        "AccountCell",
        input_account_index,
        output_account_index,
        fee,
        storage_capacity,
    )?;

    Ok(())
}

fn verify_action_throttle<'a>(
    action: Action,
    config: ConfigCellAccountReader,
    input_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    current_timestamp: u64,
) -> Result<(), Box<dyn ScriptError>> {
    macro_rules! assert_action_throttle {
        ($input_witness_reader:expr, $output_witness_reader:expr, $config_field:ident, $field:ident, $field_name:expr) => {{
            let throttle = u32::from(config.$config_field()) as u64;
            let prev = u64::from($input_witness_reader.$field());
            let current = u64::from($output_witness_reader.$field());

            if prev != 0 {
                das_assert!(
                    current >= prev + throttle,
                    AccountCellErrorCode::AccountCellThrottle,
                    "The AccountCell is used too often, need to wait {} seconds between each transaction.(current: {}, prev: {})",
                    throttle,
                    current,
                    prev
                );
            }

            das_assert!(
                current_timestamp == current,
                AccountCellErrorCode::AccountCellThrottle,
                "The AccountCell.{} in outputs should be the same as the timestamp in the TimeCell.(expected: {}, current: {})",
                $field_name,
                current_timestamp,
                current
            );
        }};
    }

    if input_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        match action {
            Action::TransferAccount => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                transfer_account_throttle,
                last_transfer_account_at,
                "last_transfer_account_at"
            ),
            Action::EditManager => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                edit_manager_throttle,
                last_edit_manager_at,
                "last_edit_manager_at"
            ),
            Action::EditRecords => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                edit_records_throttle,
                last_edit_records_at,
                "last_edit_records_at"
            ),
            _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
        }
    }

    Ok(())
}

fn verify_account_is_locked_for_cross_chain<'a>(
    output_account_index: usize,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
    current_timestamp: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Check if the AccountCell is corrently locked for cross chain.");

    let source = Source::Output;

    let data = util::load_cell_data(output_account_index, source)?;
    let expired_at = data_parser::account_cell::get_expired_at(data.as_slice());
    das_assert!(
        current_timestamp + 30 * DAY_SEC <= expired_at,
        ErrorCode::CrossChainLockError,
        "outputs[{}] Current time should be 30 days(in seconds) earlier than the AccountCell.expired_at.(current_timestamp: {}, expired_at: {})",
        output_account_index,
        current_timestamp,
        expired_at
    );

    if output_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        let account_cell_status = u8::from(output_witness_reader.status());

        das_assert!(
            account_cell_status == AccountStatus::LockedForCrossChain as u8,
            ErrorCode::CrossChainLockError,
            "outputs[{}]The AccountCell.witness.status should be LockedForCrossChain .",
            output_account_index
        );
    }

    Ok(())
}

fn verify_account_is_unlocked_for_cross_chain<'a>(
    output_account_index: usize,
    output_witness_reader: &Box<dyn AccountCellDataReaderMixer + 'a>,
) -> Result<(), Box<dyn ScriptError>> {
    if output_witness_reader.version() <= 1 {
        // CAREFUL! The early versions will no longer be supported.
        return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
    } else {
        let account_cell_status = u8::from(output_witness_reader.status());

        das_assert!(
            account_cell_status == AccountStatus::Normal as u8,
            ErrorCode::CrossChainUnlockError,
            "outputs[{}]The AccountCell.witness.status should be Normal .",
            output_account_index
        );
    }

    Ok(())
}
