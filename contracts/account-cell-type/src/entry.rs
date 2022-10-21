use alloc::boxed::Box;
use alloc::vec;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::*;
use ckb_std::dynamic_loading_c_impl::CKBDLContext;
use ckb_std::high_level;
use das_core::constants::*;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert as das_assert, code_to_error, data_parser, debug, sign_util, util, verifiers, warn};
use das_dynamic_libs::constants::{DymLibSize, CKB_MULTI_LIB_CODE_HASH};
use das_dynamic_libs::sign_lib::{SignLib, SignLibWith1Methods};
use das_map::map::Map;
use das_map::util as map_util;
use das_types::constants::{AccountStatus, SubAccountEnableStatus};
use das_types::mixer::*;
use das_types::packed::*;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    if action != b"init_account_chain" {
        util::is_system_off(&parser)?;
    }

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );
    match action {
        b"init_account_chain" => {
            unreachable!();
        }
        b"transfer_account" | b"edit_manager" | b"edit_records" | b"lock_account_for_cross_chain" => {
            verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("AccountCell", &input_account_cells, 1, &output_account_cells, 1)?;

            debug!("Verify if there is no redundant cells in inputs.");

            let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
            verifiers::misc::verify_no_more_cells_with_same_lock(
                sender_lock.as_reader(),
                &input_account_cells,
                Source::Input,
            )?;

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            let config_account = parser.configs.account()?;

            verify_transaction_fee_spent_correctly(
                action,
                config_account,
                input_account_cells[0],
                output_account_cells[0],
            )?;
            if action != b"lock_account_for_cross_chain" {
                verify_action_throttle(
                    action,
                    config_account,
                    &input_cell_witness_reader,
                    &output_cell_witness_reader,
                    timestamp,
                )?;
            }

            verifiers::account_cell::verify_status(
                &input_cell_witness_reader,
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

            match action {
                b"transfer_account" => {
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
                }
                b"edit_manager" => {
                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        Some("manager"),
                        vec![],
                        vec!["last_edit_manager_at"],
                    )?;
                }
                b"edit_records" => {
                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        None,
                        vec![],
                        vec!["records", "last_edit_records_at"],
                    )?;
                    verifiers::account_cell::verify_records_keys(&parser, output_cell_witness_reader.records())?;
                }
                b"lock_account_for_cross_chain" => {
                    verifiers::account_cell::verify_account_cell_consistent_with_exception(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        None,
                        vec![],
                        vec!["status"],
                    )?;

                    verify_account_is_locked_for_cross_chain(
                        output_account_cells[0],
                        &output_cell_witness_reader,
                        timestamp,
                    )?;
                }
                _ => unreachable!(),
            }

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        b"renew_account" => {
            parser.parse_cell()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let prices = parser.configs.price()?.prices();
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("AccountCell", &input_account_cells, 1, &output_account_cells, 1)?;

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
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
            let das_wallet_lock = Script::from(das_wallet_lock());

            let (input_income_cells, output_income_cells) = util::find_cells_by_type_id_in_inputs_and_outputs(
                ScriptType::Type,
                config_main.type_id_table().income_cell(),
            )?;

            let mut exist_capacity = 0;
            if input_income_cells.len() == 1 {
                let input_income_cell_witness =
                    util::parse_income_cell_witness(&parser, input_income_cells[0], Source::Input)?;
                let input_income_cell_witness_reader = input_income_cell_witness.as_reader();

                for item in input_income_cell_witness_reader.records().iter() {
                    if util::is_reader_eq(item.belong_to(), das_wallet_lock.as_reader()) {
                        exist_capacity += u64::from(item.capacity());
                    }
                }
            }

            let output_income_cell_witness =
                util::parse_income_cell_witness(&parser, output_income_cells[0], Source::Output)?;
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
            verifiers::income_cell::verify_income_cells(&parser, profit_map)?;

            debug!("Check if the renewal duration is longer than or equal to one year.");

            let input_data = util::load_cell_data(input_account_cells[0], Source::Input)?;
            let output_data = util::load_cell_data(output_account_cells[0], Source::Output)?;
            let input_expired_at = data_parser::account_cell::get_expired_at(&input_data);
            let output_expired_at = data_parser::account_cell::get_expired_at(&output_data);
            let duration = output_expired_at - input_expired_at;

            das_assert!(
                duration >= 365 * 86400,
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

            let yearly_capacity = util::calc_yearly_capacity(renew_price_in_usd, quote, 0);
            das_assert!(
                paid >= yearly_capacity,
                AccountCellErrorCode::AccountCellRenewDurationMustLongerThanYear,
                "The paid capacity should be at least 1 year. (current: {}, expected: >= {}",
                paid,
                yearly_capacity
            );

            // Renew price for 1 year in CKB = x ÷ y .
            let expected_duration = util::calc_duration_from_paid(paid, renew_price_in_usd, quote, 0);
            // The duration can be floated within the range of one day.
            das_assert!(
                duration >= expected_duration - 86400 && duration <= expected_duration + 86400,
                AccountCellErrorCode::AccountCellRenewDurationBiggerThanPayed,
                "The duration should be equal to {} +/- 86400s. (current: duration({}), calculation: (paid({}) / (renew_price({}) / quote({}) * 100_000_000) ) * 86400 * 365)",
                expected_duration,
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
        b"confirm_proposal" => {
            util::require_type_script(
                &parser,
                TypeScript::ProposalCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"recycle_expired_account" => {
            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_cells,
                &[0, 1],
                &output_cells,
                &[0],
            )?;

            let input_prev_cell_witness = util::parse_account_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_prev_cell_witness_reader = input_prev_cell_witness.as_reader();
            let output_prev_cell_witness = util::parse_account_cell_witness(&parser, output_cells[0], Source::Output)?;
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
            let expired_account_witness = util::parse_account_cell_witness(&parser, input_cells[1], Source::Input)?;
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
            match expired_account_witness_reader.try_into_latest() {
                Ok(reader) => {
                    let enable_sub_account = u8::from(reader.enable_sub_account());
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
                }
                _ => {}
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

                let das_wallet_lock = das_wallet_lock();
                let das_wallet_cells =
                    util::find_cells_by_script(ScriptType::Lock, das_wallet_lock.as_reader(), Source::Output)?;
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
        b"start_account_sale" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountSaleCellType,
                Source::Output,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"cancel_account_sale" | b"buy_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountSaleCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"accept_offer" => {
            util::require_type_script(
                &parser,
                TypeScript::OfferCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"force_recover_account_status" => {
            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position("AccountCell", &input_cells, &[0], &output_cells, &[0])?;

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness = util::parse_account_cell_witness(&parser, output_cells[0], Source::Output)?;
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

                let type_id = parser.configs.main()?.type_id_table().account_sale_cell();
                let (input_sale_cells, output_sale_cells) =
                    util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, type_id)?;
                verifiers::common::verify_cell_number_and_position(
                    "AccountSaleCell",
                    &input_sale_cells,
                    &[1],
                    &output_sale_cells,
                    &[],
                )?;

                let cell_witness = util::parse_account_sale_cell_witness(&parser, input_sale_cells[0], Source::Input)?;
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

            let expected_capacity = capacity_should_recycle - 10_000;
            let current_capacity = high_level::load_cell_capacity(outputs_balance_cells[0], Source::Output)?;
            das_assert!(
                current_capacity >= expected_capacity,
                ErrorCode::AccountSaleCellRefundError,
                "The capacity refunding is incorrect.(expected: {}, current: {})",
                expected_capacity,
                current_capacity
            );
        }
        b"enable_sub_account" => {
            // CAREFUL! This action is intentionally ignoring EIP712 verification.
            // verify_eip712_hashes(&parser, transfer_account_to_semantic)?;

            verifiers::account_cell::verify_unlock_role(action, &parser.params)?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            das_assert!(
                input_account_cells.len() == 1 && input_account_cells[0] == 0,
                ErrorCode::InvalidTransactionStructure,
                "There should be one AccountCell at inputs[0]."
            );
            das_assert!(
                output_account_cells.len() == 1 && output_account_cells[0] == 0,
                ErrorCode::InvalidTransactionStructure,
                "There should bze one AccountCell at outputs[0]."
            );

            let input_account_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_witness_reader = input_account_witness.as_reader();
            let output_account_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_witness_reader = output_account_witness.as_reader();

            let account = util::get_account_from_reader(&input_account_witness_reader);
            verifiers::sub_account_cell::verify_beta_list(&parser, account.as_bytes())?;

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

            match input_account_witness_reader.try_into_latest() {
                Ok(reader) => {
                    let enable_status = u8::from(reader.enable_sub_account());
                    das_assert!(
                        enable_status == SubAccountEnableStatus::Off as u8,
                        AccountCellErrorCode::AccountCellPermissionDenied,
                        "{:?}[{}] Only AccountCells with enable_sub_account field is {} can enable its sub-account function.",
                        Source::Input,
                        input_account_cells[0],
                        SubAccountEnableStatus::Off as u8
                    );
                }
                Err(_) => {
                    // If the AccountCell in inputs is old version, it definitely have not enabled the sub-account function.
                }
            }

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
                        "{:?}[{}]The version of this AccountCell should be latest.",
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
                ErrorCode::SubAccountCellCapacityError,
                "The initial capacity of SubAccountCell should be equal to ConfigCellSubAccount.basic_capacity + ConfigCellSubAccount.prepared_fee_capacity .(expected: {}, current: {})",
                expected_capacity,
                sub_account_cell_capacity
            );

            let type_script = high_level::load_cell_type(output_sub_account_cells[0], Source::Output)?.unwrap();
            let account_id = type_script.as_reader().args().raw_data();
            let expected_account_id = output_account_witness_reader.id().raw_data();

            das_assert!(
                account_id == expected_account_id,
                ErrorCode::SubAccountCellAccountIdError,
                "The type.args of SubAccountCell should be the same with the AccountCell.witness.id .(expected: {}, current: {})",
                util::hex_string(expected_account_id),
                util::hex_string(account_id)
            );

            let sub_account_outputs_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;
            let expected_default_data = vec![0u8; 48];

            das_assert!(
                expected_default_data == sub_account_outputs_data,
                ErrorCode::SubAccountCellSMTRootError,
                "The default outputs_data of SubAccountCell should be [0u8; 48] ."
            );

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
        b"create_sub_account" => {
            util::require_type_script(
                &parser,
                TypeScript::SubAccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"config_sub_account_custom_script" => {
            util::require_type_script(
                &parser,
                TypeScript::SubAccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"unlock_account_for_cross_chain" => {
            parser.parse_cell()?;

            debug!("Verify if there is no redundant AccountCells.");

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[0],
                &output_account_cells,
                &[0],
            )?;

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            let config_account = parser.configs.account()?;

            // include: common::verify_tx_fee_spent_correctly
            verify_transaction_fee_spent_correctly(
                action,
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
            // CAREFUL! The owner lock may be changed or not changed, only the keepers know it, so we skip verification here.
            match verifiers::account_cell::verify_account_lock_consistent(
                input_account_cells[0],
                output_account_cells[0],
                None,
            ) {
                Ok(_) => {
                    // The lock is not changed, so the records must be kept.
                    verifiers::account_cell::verify_account_witness_consistent(
                        input_account_cells[0],
                        output_account_cells[0],
                        &input_cell_witness_reader,
                        &output_cell_witness_reader,
                        vec!["status"],
                    )?;
                }
                Err(err) => {
                    if err.as_i8() == ErrorCode::CellLockCanNotBeModified as i8 {
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
                        return Err(err);
                    }
                }
            }

            verify_account_is_unlocked_for_cross_chain(output_account_cells[0], &output_cell_witness_reader)?;

            verify_multi_sign(input_account_cells[0])?;
        }
        b"confirm_expired_account_auction" => {
            parser.parse_cell()?;

            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

            debug!("Verify if there is no redundant AccountCells.");

            let (input_account_cells, output_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "AccountCell",
                &input_account_cells,
                &[0],
                &output_account_cells,
                &[0],
            )?;

            let input_cell_witness = util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_cell_witness_reader = input_cell_witness.as_reader();
            let output_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_cell_witness_reader = output_cell_witness.as_reader();

            // include: common::verify_tx_fee_spent_correctly
            verify_transaction_fee_spent_correctly(
                action,
                config_account,
                input_account_cells[0],
                output_account_cells[0],
            )?;

            // Verify if the expired account auction is ended.
            match verifiers::account_cell::verify_account_expiration(
                config_account,
                input_account_cells[0],
                Source::Input,
                timestamp,
            ) {
                Ok(_) => {
                    warn!("The AccountCell is not expired.");
                    return Err(code_to_error!(AccountCellErrorCode::AccountCellIsNotExpired));
                }
                Err(err) => {
                    if err.as_i8() == AccountCellErrorCode::AccountCellInExpirationGracePeriod as i8 {
                        warn!("The AccountCell is not expired.");
                        return Err(code_to_error!(AccountCellErrorCode::AccountCellIsNotExpired));
                    } else if err.as_i8() == AccountCellErrorCode::AccountCellInExpirationAuctionPeriod as i8 {
                        warn!("The AccountCell is still in auction period.");
                        return Err(code_to_error!(
                            AccountCellErrorCode::AccountCellInExpirationAuctionPeriod
                        ));
                    } else {
                        // Ok
                    }
                }
            }

            verifiers::account_cell::verify_status(
                &input_cell_witness_reader,
                AccountStatus::Normal,
                input_account_cells[0],
                Source::Input,
            )?;
            verifiers::account_cell::verify_account_data_consistent(
                input_account_cells[0],
                output_account_cells[0],
                vec![],
            )?;
            // Even the lock has not been changed, the records still need to be cleared.
            verifiers::account_cell::verify_account_witness_consistent(
                input_account_cells[0],
                output_account_cells[0],
                &input_cell_witness_reader,
                &output_cell_witness_reader,
                vec!["records"],
            )?;
            verifiers::account_cell::verify_account_witness_record_empty(
                &output_cell_witness_reader,
                output_account_cells[0],
                Source::Output,
            )?;

            verify_multi_sign(input_account_cells[0])?;

            debug!("Verify if the SubAccountCell has been refund properly.");

            let mut refund_from_sub_account_cell_to_das = 0;
            let mut refund_from_sub_account_cell_to_owner = 0;
            match input_cell_witness_reader.try_into_latest() {
                Ok(reader) => {
                    let enable_sub_account = u8::from(reader.enable_sub_account());
                    if enable_sub_account == SubAccountEnableStatus::On as u8 {
                        debug!("Verify if the SubAccountCell is refunded properly.");

                        let config_sub_account = parser.configs.sub_account()?;
                        let basic_capacity = u64::from(config_sub_account.basic_capacity());

                        let sub_account_type_id = config_main.type_id_table().sub_account_cell();
                        let (input_sub_account_cells, output_sub_account_cells) =
                            util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, sub_account_type_id)?;

                        verifiers::common::verify_cell_number_and_position(
                            "SubAccountCell",
                            &input_sub_account_cells,
                            &[1],
                            &output_sub_account_cells,
                            &[1],
                        )?;

                        verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                            input_sub_account_cells[0],
                            output_sub_account_cells[0],
                            vec!["das_profit", "owner_profit"],
                        )?;

                        // For simplicity, the capacity of the SubAccountCell in inputs is ignored.
                        let output_sub_account_capacity =
                            high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;

                        das_assert!(
                            output_sub_account_capacity == basic_capacity,
                            ErrorCode::InvalidTransactionStructure,
                            "outputs[{}] The capacity of the SubAccountCell should be {} shannon.",
                            output_sub_account_cells[0],
                            basic_capacity
                        );

                        let input_sub_account_data =
                            high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
                        let output_sub_account_data =
                            high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;
                        let input_das_profit =
                            data_parser::sub_account_cell::get_das_profit(&input_sub_account_data).unwrap();
                        let output_das_profit =
                            data_parser::sub_account_cell::get_das_profit(&output_sub_account_data).unwrap();
                        let input_owner_profit =
                            data_parser::sub_account_cell::get_owner_profit(&input_sub_account_data).unwrap();
                        let output_owner_profit =
                            data_parser::sub_account_cell::get_owner_profit(&output_sub_account_data).unwrap();

                        das_assert!(
                            output_das_profit == 0 && output_owner_profit == 0,
                            ErrorCode::SubAccountCollectProfitError,
                            "All profit in the SubAccountCell should be collected."
                        );

                        refund_from_sub_account_cell_to_owner = input_owner_profit;
                        refund_from_sub_account_cell_to_das = input_das_profit;
                    }
                }
                _ => {}
            }

            debug!("Verify if all the refunds has been refund properly.");

            let expired_account_capacity = high_level::load_cell_capacity(input_account_cells[0], Source::Input)?;
            let refund_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;

            verifiers::misc::verify_user_get_change(
                config_main,
                refund_lock.as_reader(),
                expired_account_capacity + refund_from_sub_account_cell_to_owner,
            )?;

            if refund_from_sub_account_cell_to_das >= CELL_BASIC_CAPACITY {
                verifiers::common::verify_das_get_change(refund_from_sub_account_cell_to_das)?;
            } else {
                debug!(
                    "The profit of DAS is {} shannon, so no need to refund to DAS.",
                    refund_from_sub_account_cell_to_das
                );
            }
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}

fn verify_transaction_fee_spent_correctly(
    action: &[u8],
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
        b"transfer_account" => u64::from(config.transfer_account_fee()),
        b"edit_manager" => u64::from(config.edit_manager_fee()),
        b"edit_records" => u64::from(config.edit_records_fee()),
        _ => u64::from(config.common_fee()),
    };
    let storage_capacity = basic_capacity + account_length * 100_000_000;

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
    action: &[u8],
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
            b"transfer_account" => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                transfer_account_throttle,
                last_transfer_account_at,
                "last_transfer_account_at"
            ),
            b"edit_manager" => assert_action_throttle!(
                input_witness_reader,
                output_witness_reader,
                edit_manager_throttle,
                last_edit_manager_at,
                "last_edit_manager_at"
            ),
            b"edit_records" => assert_action_throttle!(
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

fn verify_multi_sign(input_account_index: usize) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the signatures of secp256k1-blake160-multisig-all ...");

    let (digest, _, witness_args_lock) =
        sign_util::calc_digest_by_input_group(SignType::Secp256k1Blake160MultiSigAll, vec![input_account_index])?;
    let lock_script = cross_chain_lock();
    let mut args = lock_script.as_reader().args().raw_data().to_vec();
    let since = high_level::load_input_since(input_account_index, Source::Input)?;

    // It is the signature validation requirement.
    args.extend_from_slice(&since.to_le_bytes());

    debug!(
        "Loading dynamic library by code_hash: 0x{}",
        util::hex_string(&CKB_MULTI_LIB_CODE_HASH)
    );

    if cfg!(not(feature = "dev")) {
        let mut context = unsafe { CKBDLContext::<DymLibSize>::new() };
        let lib = context
            .load(&CKB_MULTI_LIB_CODE_HASH)
            .expect("The shared lib should be loaded successfully.");
        let methods = SignLibWith1Methods {
            c_validate: unsafe {
                lib.get(b"validate")
                    .expect("Load function 'validate' from library failed.")
            },
        };
        let sign_lib = SignLib::new(None, None, Some(methods));

        sign_lib
            .validate(DasLockType::CKBMulti, 0i32, digest.to_vec(), witness_args_lock, args)
            .map_err(|err_code| {
                warn!(
                    "inputs[{}] Verify signature failed, error code: {}",
                    input_account_index, err_code
                );
                return ErrorCode::EIP712SignatureError;
            })?;
    }

    Ok(())
}
