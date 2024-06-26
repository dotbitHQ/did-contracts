use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed;
use ckb_std::high_level;
use das_core::config::Config;
use das_core::constants::*;
use das_core::error::*;
use das_core::util::{self, exec_das_lock};
use das_core::witness_parser::sub_account::*;
use das_core::{assert as das_assert, code_to_error, data_parser, debug, verifiers, warn};
use das_types::constants::{
    das_lock, profit_manager_lock, signhash_lock, AccountStatus, Action, DataType, LockRole, SubAccountConfigFlag,
    SubAccountCustomRuleFlag, TypeScript,
};
use das_types::packed::*;
use das_types::prelude::{Builder, Entity};
use simple_ast::executor::match_rule_with_account_chars;
use witness_parser::WitnessesParserV1;

use crate::sub_action::SubAction;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running sub-account-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser
        .init()
        .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

    util::is_system_off()?;

    debug!("Route to {:?} action ...", parser.action.to_string());
    match parser.action {
        Action::EnableSubAccount => {
            util::require_type_script(
                TypeScript::AccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::RecycleExpiredAccount => {
            util::require_type_script(
                TypeScript::AccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        Action::ConfigSubAccount => action_config_sub_account()?,
        Action::UpdateSubAccount => action_update_sub_account()?,
        Action::CollectSubAccountProfit | Action::CollectSubAccountChannelProfit => {
            action_collect_sub_account_profit()?
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}

fn action_config_sub_account() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let config_account = Config::get_instance().account()?;
    let config_sub_account = Config::get_instance().sub_account()?;

    let timestamp = util::load_oracle_data(OracleCellType::Time)?;

    debug!("Verify if the AccountCell is consistent and not expired ...");

    let (input_account_cells, output_account_cells) = util::find_cells_by_type_id_in_inputs_and_outputs(
        ScriptType::Type,
        config_main.type_id_table().account_cell(),
    )?;
    verifiers::common::verify_cell_number_and_position(
        "AccountCell",
        &input_account_cells,
        &[0],
        &output_account_cells,
        &[0],
    )?;

    let sender_lock = util::derive_owner_lock_from_cell(input_account_cells[0], Source::Input)?;
    verifiers::misc::verify_no_more_cells_with_same_lock(sender_lock.as_reader(), &input_account_cells, Source::Input)?;

    let input_account_cell_witness = util::parse_account_cell_witness(input_account_cells[0], Source::Input)?;
    let input_account_cell_reader = input_account_cell_witness.as_reader();
    let output_account_cell_witness = util::parse_account_cell_witness(output_account_cells[0], Source::Output)?;
    let output_account_cell_reader = output_account_cell_witness.as_reader();

    verifiers::account_cell::verify_status_v2(
        &input_account_cell_reader,
        &[AccountStatus::Normal, AccountStatus::ApprovedTransfer],
        input_account_cells[0],
        Source::Input,
    )?;

    verifiers::account_cell::verify_account_expiration(
        config_account,
        input_account_cells[0],
        Source::Input,
        timestamp,
    )?;

    verifiers::account_cell::verify_account_capacity_not_decrease(input_account_cells[0], output_account_cells[0])?;

    verifiers::account_cell::verify_account_cell_consistent_with_exception(
        input_account_cells[0],
        output_account_cells[0],
        &input_account_cell_reader,
        &output_account_cell_reader,
        None,
        vec![],
        vec![],
    )?;

    debug!("Verify if the SubAccountCell is consistent and only the appropriate fee was charged ...");

    let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    verifiers::common::verify_cell_number_and_position(
        "SubAccountCell",
        &input_sub_account_cells,
        &[1],
        &output_sub_account_cells,
        &[1],
    )?;

    verifiers::sub_account_cell::verify_sub_account_parent_id(
        input_sub_account_cells[0],
        Source::Input,
        input_account_cell_reader.id().raw_data(),
    )?;

    let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
    let output_sub_account_capacity = high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
    let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
    let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

    verify_sub_account_transaction_fee(
        config_sub_account,
        input_sub_account_capacity,
        &input_sub_account_data,
        output_sub_account_capacity,
        &output_sub_account_data,
    )?;

    verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
        input_sub_account_cells[0],
        output_sub_account_cells[0],
        vec!["flag", "custom_rule_status_flag", "price_rules", "preserved_rules"],
    )?;

    debug!("Verify if the config fields is updated appropriately ...");

    let sub_account_cell_data = util::load_cell_data(output_sub_account_cells[0], Source::Output)?;
    let flag = match data_parser::sub_account_cell::get_flag(&output_sub_account_data) {
        Some(val) => val,
        None => {
            warn!(
                "The flag is invalid.(sub_account_data: {})",
                util::hex_string(&output_sub_account_data)
            );
            return Err(code_to_error!(SubAccountCellErrorCode::ConfigFlagInvalid));
        }
    };
    let price_rules_hash = data_parser::sub_account_cell::get_price_rules_hash(&sub_account_cell_data);
    let preserved_rules_hash = data_parser::sub_account_cell::get_preserved_rules_hash(&sub_account_cell_data);

    match flag {
        SubAccountConfigFlag::CustomRule => {
            verifiers::sub_account_cell::verify_config_is_custom_rule(output_sub_account_cells[0], Source::Output)?;

            let mut rules_to_verify = vec![];
            if price_rules_hash != Some(&[0u8; 10]) {
                rules_to_verify.push(DataType::SubAccountPriceRule);
            }
            if preserved_rules_hash != Some(&[0u8; 10]) {
                rules_to_verify.push(DataType::SubAccountPreservedRule);
            }

            if !rules_to_verify.is_empty() {
                let sub_account_witness_parser = SubAccountWitnessesParser::new(flag, &config_main)?;
                for data_type in rules_to_verify {
                    let (hash, field) = match data_type {
                        DataType::SubAccountPriceRule => (price_rules_hash, String::from("price_rules")),
                        DataType::SubAccountPreservedRule => (preserved_rules_hash, String::from("preserved_rules")),
                        _ => unreachable!(),
                    };

                    let rules = match sub_account_witness_parser.get_rules(&sub_account_cell_data, data_type)? {
                        (_, Some(rules)) => rules,
                        (_, None) => {
                            das_assert!(
                                hash == Some(&[0u8; 10]),
                                SubAccountCellErrorCode::ConfigRulesHashMismatch,
                                "The {}Witness is empty, but the SubAccountCell.data.{}_hash is not 0x00000000000000000000",
                                data_type.to_string(),
                                field
                            );

                            continue;
                        }
                    };

                    if data_type == DataType::SubAccountPriceRule {
                        for rule in rules.iter() {
                            das_assert!(
                                rule.price >= u64::from(config_sub_account.new_sub_account_price()),
                                SubAccountCellErrorCode::ConfigRulesPriceError,
                                "The SubAccountCell.witness.{} has price error, the minimal price should be {} in USD .",
                                field,
                                config_sub_account.new_sub_account_price()
                            );
                        }
                    }

                    let mut dummy_account_chars_builder = AccountChars::new_builder();
                    dummy_account_chars_builder = dummy_account_chars_builder.push(AccountChar::default());
                    let dummy_account_chars = dummy_account_chars_builder.build();
                    let dummy_account = "";

                    match_rule_with_account_chars(&rules, dummy_account_chars.as_reader(), dummy_account).map_err(
                        |err| {
                            warn!(
                                "The SubAccountCell.witness.{} has some syntax error: {}",
                                field,
                                err.to_string()
                            );
                            code_to_error!(SubAccountCellErrorCode::ConfigRulesHasSyntaxError)
                        },
                    )?;
                }
            } else {
                debug!("No rules configured, skip the syntax check ...");
            }
        }
        SubAccountConfigFlag::Manual => {
            verifiers::sub_account_cell::verify_config_is_manual(output_sub_account_cells[0], Source::Output)?;
        }
        _ => {
            warn!("The flag CustomScript is no longer supported.");
            return Err(code_to_error!(ErrorCode::HardCodedError));
        }
    }

    Ok(())
}

fn action_update_sub_account() -> Result<(), Box<dyn ScriptError>> {
    let config_main = Config::get_instance().main()?;
    let config_account = Config::get_instance().account()?;
    let config_sub_account = Config::get_instance().sub_account()?;

    let timestamp = util::load_oracle_data(OracleCellType::Time)?;
    let quote = util::load_oracle_data(OracleCellType::Quote)?;

    debug!("Preparing to parse sub-account witnesses by loading the SubAccountCell ...");

    let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

    verifiers::common::verify_cell_number_and_position(
        "SubAccountCell",
        &input_sub_account_cells,
        &[0],
        &output_sub_account_cells,
        &[0],
    )?;

    let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
    let output_sub_account_capacity = high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
    let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
    let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

    let flag = match data_parser::sub_account_cell::get_flag(&output_sub_account_data) {
        Some(val) => val,
        None => {
            warn!(
                "The flag is invalid.(sub_account_data: {})",
                util::hex_string(&output_sub_account_data)
            );
            return Err(code_to_error!(SubAccountCellErrorCode::ConfigFlagInvalid));
        }
    };
    let sub_account_parser = SubAccountWitnessesParser::new(flag, &config_main)?;

    debug!("Verify if the AccountCell in cell_deps has sub-account feature enabled and not expired ...");

    let dep_account_cells = util::find_cells_by_type_id(
        ScriptType::Type,
        config_main.type_id_table().account_cell(),
        Source::CellDep,
    )?;

    verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

    let account_cell_index = dep_account_cells[0];
    let account_cell_source = Source::CellDep;
    let account_cell_witness = util::parse_account_cell_witness(dep_account_cells[0], Source::CellDep)?;
    let account_cell_reader = account_cell_witness.as_reader();
    let account_cell_data = util::load_cell_data(account_cell_index, account_cell_source)?;
    let account_lock = high_level::load_cell_lock(account_cell_index, account_cell_source)?;
    let account_lock_args = account_lock.as_reader().args().raw_data();

    verifiers::account_cell::verify_sub_account_enabled(&account_cell_reader, account_cell_index, account_cell_source)?;

    if sub_account_parser.only_contains_recycle() {
        debug!("This transaction only contains recycle action, skip the account expiration check ...");
    } else {
        verifiers::account_cell::verify_account_expiration(
            config_account,
            account_cell_index,
            account_cell_source,
            timestamp,
        )?;
    }

    let mut parent_account = account_cell_reader.account().as_readable();
    parent_account.extend(ACCOUNT_SUFFIX.as_bytes());

    debug!("Verify if the SubAccountCells have sufficient capacity and paid transaction fees properly ...");

    verify_sub_account_capacity_is_enough(
        config_sub_account,
        input_sub_account_cells[0],
        input_sub_account_capacity,
        &input_sub_account_data,
        output_sub_account_cells[0],
        output_sub_account_capacity,
        &output_sub_account_data,
    )?;

    let is_fee_paied = verify_sub_account_transaction_fee(
        config_sub_account,
        input_sub_account_capacity,
        &input_sub_account_data,
        output_sub_account_capacity,
        &output_sub_account_data,
    )?;

    verifiers::sub_account_cell::verify_sub_account_parent_id(
        input_sub_account_cells[0],
        Source::Input,
        account_cell_reader.id().raw_data(),
    )?;

    debug!("Initialize some vars base on the sub-actions contains in the transaction ...");

    let parent_expired_at = data_parser::account_cell::get_expired_at(&account_cell_data);
    let header = util::load_header(input_sub_account_cells[0], Source::Input)?;
    let sub_account_last_updated_at = util::get_timestamp_from_header(header.as_reader());

    let mut manual_mint_list_smt_root = None;
    let mut manual_renew_list_smt_root = None;
    let mut sender_lock = packed::Script::default();

    let mut custom_rule_flag = SubAccountCustomRuleFlag::Off;
    let mut custom_price_rules = None;
    let mut is_custom_price_rules_set = false;
    let mut custom_preserved_rules = None;

    let mut smt_root_sign_found = false;
    if sub_account_parser.contains_creation || sub_account_parser.contains_renew {
        debug!("Found `create/renew` action in this transaction, do some common verfications ...");

        let extract_smt_root_sign_witness =
            |_name: &str,
             witness: &SubAccountMintSignWitness|
             -> Result<(Option<LockRole>, packed::Script, Option<[u8; 32]>), Box<dyn ScriptError>> {
                debug!("The {} is exist, verifying the signature for manual mint ...", _name);

                let mut tmp = [0u8; 32];
                tmp.copy_from_slice(&witness.account_list_smt_root);
                let account_list_smt_root = Some(tmp);

                let sender_lock = if witness.sign_role == Some(LockRole::Manager) {
                    debug!("Found SubAccountWitness.sign_role is manager, use manager lock as sender_lock.");
                    util::derive_manager_lock_from_cell(account_cell_index, account_cell_source)?
                } else {
                    debug!("Found SubAccountWitness.sign_role is owner, use owner lock as sender_lock.");
                    util::derive_owner_lock_from_cell(account_cell_index, account_cell_source)?
                };

                Ok((witness.sign_role.clone(), sender_lock, account_list_smt_root))
            };

        let mut mint_sign_role = None;
        if sub_account_parser.contains_creation {
            match sub_account_parser.get_mint_sign(account_lock_args) {
                Some(Ok(witness)) => {
                    smt_root_sign_found = true;
                    (mint_sign_role, sender_lock, manual_mint_list_smt_root) =
                        extract_smt_root_sign_witness("SubAccountMintWitness", &witness)?;
                }
                Some(Err(err)) => {
                    return Err(err);
                }
                None => {
                    debug!("There is no SubAccountMintSign found.");
                }
            }
        }

        if sub_account_parser.contains_renew {
            match sub_account_parser.get_renew_sign(account_lock_args) {
                Some(Ok(witness)) => {
                    let renew_sender_lock;
                    let renew_sign_role;
                    smt_root_sign_found = true;
                    (renew_sign_role, renew_sender_lock, manual_renew_list_smt_root) =
                        extract_smt_root_sign_witness("SubAccountRenewWitness", &witness)?;

                    if mint_sign_role.is_some() {
                        das_assert!(
                            mint_sign_role == renew_sign_role,
                            SubAccountCellErrorCode::MultipleSignRolesIsNotAllowed,
                            "The sign_role of SubAccountMintSignWitness and SubAccountRenewSignWitness should be the same in the same transaction."
                        );
                    } else {
                        sender_lock = renew_sender_lock;
                    }
                }
                Some(Err(err)) => {
                    return Err(err);
                }
                None => {
                    debug!("There is no SubAccountRenewSign found.");
                }
            }
        } else {
            if sub_account_parser.get_renew_sign(account_lock_args).is_some() {
                warn!("The SubAccountRenewSignWitness is not allowed if there if no renew action exists.");
                return Err(code_to_error!(SubAccountCellErrorCode::SubAccountRenewSignIsNotAllowed));
            }
        }

        debug!("The SubAccountCell.data.flag is {} .", flag.to_string());

        match flag {
            SubAccountConfigFlag::CustomRule => {
                verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                    input_sub_account_cells[0],
                    output_sub_account_cells[0],
                    vec!["smt_root", "das_profit"],
                )?;

                debug!("Parsing custom rules from witness ...");

                custom_rule_flag =
                    match data_parser::sub_account_cell::get_custom_rule_status_flag(&input_sub_account_data) {
                        Some(val) => val,
                        None => SubAccountCustomRuleFlag::Off,
                    };
                (is_custom_price_rules_set, custom_price_rules) =
                    sub_account_parser.get_rules(&input_sub_account_data, DataType::SubAccountPriceRule)?;
                (_, custom_preserved_rules) =
                    sub_account_parser.get_rules(&input_sub_account_data, DataType::SubAccountPreservedRule)?;
            }
            SubAccountConfigFlag::Manual => {
                verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                    input_sub_account_cells[0],
                    output_sub_account_cells[0],
                    vec!["smt_root", "das_profit"],
                )?;
            }
            SubAccountConfigFlag::CustomScript => {
                warn!("The flag CustomScript is no longer supported.");
                return Err(code_to_error!(ErrorCode::HardCodedError));
            }
        }

        // The AccountCell can be in both normal or approved_transfer status.
        verifiers::account_cell::verify_status_v2(
            &account_cell_reader,
            &[AccountStatus::Normal, AccountStatus::ApprovedTransfer],
            account_cell_index,
            account_cell_source,
        )?;
    } else {
        if sub_account_parser.contains_edition || sub_account_parser.contains_recycle {
            debug!(
                "Found `edit/recycle` action in this transaction but no `create` action, so do some common verfications ..."
            );

            verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["smt_root"],
            )?;
        } else {
            debug!("No writing action found, the SubAccountCell must be consistent ...");

            verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec![],
            )?;
        }
    }

    debug!("Verify if there is any BalanceCell is abused ...");

    // CAREFUL This is very important, only update it with fully understanding the requirements.
    // This verification is also exists in das-lock.
    let das_lock = das_lock();
    let all_inputs_with_das_lock =
        util::find_cells_by_type_id(ScriptType::Lock, das_lock.code_hash().as_reader().into(), Source::Input)?;
    let mut sender_total_input_capacity = 0;
    if smt_root_sign_found {
        let dpoint_type_id = Config::get_instance().main()?.type_id_table().dpoint_cell();
        let input_sender_balance_cells = util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Input)?;

        verifiers::misc::verify_no_more_cells_with_same_lock_except_type(
            sender_lock.as_reader(),
            &input_sender_balance_cells,
            Source::Input,
            dpoint_type_id,
        )?;

        let input_sender_cells = util::find_cells_by_script(ScriptType::Lock, sender_lock.as_reader(), Source::Input)?;
        das_assert!(
            all_inputs_with_das_lock == input_sender_cells,
            SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused,
            "Some cells with das-lock have may be abused.(invalid_inputs: {:?})",
            all_inputs_with_das_lock
                .iter()
                .filter(|item| !input_sender_balance_cells.contains(item))
                .map(|item| item.to_owned())
                .collect::<Vec<usize>>()
        );

        sender_total_input_capacity = if input_sender_balance_cells.is_empty() {
            0
        } else {
            util::load_cells_capacity(&input_sender_balance_cells, Source::Input)?
        };
    } else {
        debug!("Verify if there is no BalanceCells are spent.");

        das_assert!(
            all_inputs_with_das_lock.len() == 0,
            SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused,
            "Some cells with das-lock have may be abused.(invalid_inputs: {:?})",
            all_inputs_with_das_lock
        );
    }

    debug!("Start iterating sub-account witnesses ...");

    // The first smt root is in the outputs_data of the SubAccountCell in inputs.
    let mut prev_root = match data_parser::sub_account_cell::get_smt_root(&input_sub_account_data) {
        Some(val) => val.to_vec(),
        None => {
            warn!(
                "inputs[{}] The outputs_data.smt_root should be exist.",
                input_sub_account_cells[0]
            );
            return Err(code_to_error!(ErrorCode::InvalidCellData));
        }
    };
    // The latest smt root is in the outputs_data of the SubAccountCell in outputs.
    let latest_root = match data_parser::sub_account_cell::get_smt_root(&output_sub_account_data) {
        Some(val) => val.to_vec(),
        None => {
            warn!(
                "outputs[{}] The outputs_data.smt_root should be exist.",
                output_sub_account_cells[0]
            );
            return Err(code_to_error!(ErrorCode::InvalidCellData));
        }
    };

    let mut sub_action = SubAction::new(
        //sign_lib,
        timestamp,
        quote,
        flag,
        custom_rule_flag,
        sub_account_last_updated_at,
        config_account,
        config_sub_account,
        &parent_account,
        parent_expired_at,
        &manual_mint_list_smt_root,
        &manual_renew_list_smt_root,
        &custom_preserved_rules,
        &custom_price_rules,
        is_custom_price_rules_set,
    );
    for (i, witness_ret) in sub_account_parser.iter().enumerate() {
        let witness = match witness_ret {
            Ok(val) => val,
            Err(e) => return Err(e),
        };

        sub_action.dispatch(&witness, &prev_root, &sub_account_parser)?;
        prev_root = witness.new_root.clone();

        if i == sub_account_parser.len() - 1 {
            debug!(
                "  witnesses[{:>2}] Verify if the last witness.new_root is consistent with the latest SMT root in the SubAccountCell in the outputs..",
                witness.index
            );

            let latest_root_in_witness = witness.new_root.as_slice();
            das_assert!(
                latest_root_in_witness == latest_root,
                SubAccountCellErrorCode::SubAccountWitnessMismatched,
                "  witnesses[{:>2}] The latest SMT root in witnesses should be consistent with the latest SMT root in the SubAccountCell in the outputs.(in_witness: {}, in_data: {})",
                witness.index,
                util::hex_string(latest_root_in_witness),
                util::hex_string(&latest_root)
            );
        }
    }

    if sub_account_parser.contains_creation || sub_account_parser.contains_renew {
        debug!("Verify if the profit distribution is correct.");

        let _minimal_required_das_profit = sub_action.minimal_required_das_profit;
        let profit_from_manual_mint = sub_action.profit_from_manual_mint;
        let profit_from_manual_renew = sub_action.profit_from_manual_renew;
        let profit_from_manual_renew_by_other = sub_action.profit_from_manual_renew_by_other;
        let profit_total = sub_action.profit_total;

        // debug!("minimal_required_das_profit: {:?}", minimal_required_das_profit);
        // debug!("profit_from_manual_mint: {:?}", profit_from_manual_mint);
        // debug!("profit_from_manual_renew: {:?}", profit_from_manual_renew);
        // debug!("profit_from_manual_renew_by_other: {:?}", profit_from_manual_renew_by_other);
        // debug!("profit_total: {:?}", profit_total);

        if profit_from_manual_renew_by_other > 0 {
            debug!("Found profit paied by others, verify if they only used NormalCells.");

            let lock = signhash_lock();
            let normal_cells =
                util::find_cells_by_type_id(ScriptType::Lock, lock.as_reader().code_hash().into(), Source::Input)?;
            // 0 is SubAccountCell, all_inputs_with_das_lock are BalanceCells paied by owner/manager
            let all_inputs = [vec![0], all_inputs_with_das_lock.clone(), normal_cells].concat();

            verifiers::misc::verify_no_more_cells(&all_inputs, Source::Input)?;
        }

        if sender_total_input_capacity > 0 {
            debug!("Verify if the sender capacity cost is correct.");

            let output_sender_balance_cells =
                util::find_balance_cells(config_main, sender_lock.as_reader(), Source::Output)?;
            let sender_total_output_capacity = util::load_cells_capacity(&output_sender_balance_cells, Source::Output)?;

            if sender_total_input_capacity > sender_total_output_capacity {
                let fee_to_pay = if is_fee_paied {
                    0
                } else {
                    u64::from(config_sub_account.common_fee())
                };

                // debug!("sender_total_input_capacity: {:?}", sender_total_input_capacity);
                // debug!("sender_total_output_capacity: {:?}", sender_total_output_capacity);

                das_assert!(
                    sender_total_input_capacity - sender_total_output_capacity <= profit_from_manual_mint + profit_from_manual_renew + fee_to_pay,
                    SubAccountCellErrorCode::SenderCapacityOverCost,
                    "The sender capacity cost should be <= the cost for manual mint and fee(if not paied by SubAccountCell).(should <=: {}, actual: {})",
                    profit_from_manual_mint + profit_from_manual_renew + fee_to_pay,
                    sender_total_input_capacity - sender_total_output_capacity
                );
            }
        } else {
            debug!("The sender does not have any capacity cost, skip verification.");
        }

        match flag {
            SubAccountConfigFlag::CustomRule => {
                debug!("Verify if all the profit have been accounted for .bit ...");

                verify_profit_to_das_with_custom_rule(
                    config_sub_account,
                    &input_sub_account_data,
                    &output_sub_account_data,
                    profit_total,
                )?;
            }
            SubAccountConfigFlag::Manual => {
                verify_profit_to_das_with_manual(
                    output_sub_account_cells[0],
                    &input_sub_account_data,
                    &output_sub_account_data,
                    profit_total,
                )?;
            }
            SubAccountConfigFlag::CustomScript => {
                warn!("The flag CustomScript is no longer supported.");
                return Err(code_to_error!(ErrorCode::HardCodedError));
            }
        }
    }

    debug!("Call Das-lock to complete the sub-account signature verification.");
    if all_inputs_with_das_lock.len() == 0 {
        exec_das_lock().expect("exec das-lock failed");
    }

    Ok(())
}

fn action_collect_sub_account_profit() -> Result<(), Box<dyn ScriptError>> {
    let parser = WitnessesParserV1::get_instance();
    let config_main = Config::get_instance().main()?;
    let config_sub_account = Config::get_instance().sub_account()?;

    debug!("Try to find the SubAccountCells from inputs and outputs ...");

    let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

    verifiers::common::verify_cell_number_and_position(
        "SubAccountCell",
        &input_sub_account_cells,
        &[0],
        &output_sub_account_cells,
        &[0],
    )?;

    let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
    let output_sub_account_capacity = high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
    let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
    let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;
    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_sub_account_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_sub_account_data).unwrap();
    let transaction_fee = u64::from(config_sub_account.common_fee());

    match parser.action {
        Action::CollectSubAccountProfit => {
            debug!("Try to find the AccountCell from cell_deps ...");

            let dep_account_cells = util::find_cells_by_type_id(
                ScriptType::Type,
                config_main.type_id_table().account_cell(),
                Source::CellDep,
            )?;

            verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

            let account_cell_witness = util::parse_account_cell_witness(dep_account_cells[0], Source::CellDep)?;
            let account_cell_reader = account_cell_witness.as_reader();

            verifiers::sub_account_cell::verify_sub_account_parent_id(
                input_sub_account_cells[0],
                Source::Input,
                account_cell_reader.id().raw_data(),
            )?;

            verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["das_profit", "owner_profit"],
            )?;

            let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_sub_account_data).unwrap();
            let output_owner_profit =
                data_parser::sub_account_cell::get_owner_profit(&output_sub_account_data).unwrap();

            das_assert!(
                input_das_profit > 0 || input_owner_profit > 0,
                ErrorCode::InvalidTransactionStructure,
                "Either the profit of DAS or the profit of owner should not be 0 ."
            );

            let mut collected = false;
            let mut expected_remain_capacity = input_sub_account_capacity;

            if input_das_profit > 0 && output_das_profit == 0 {
                debug!("The profit of DAS has been collected.");

                collected = true;
                expected_remain_capacity -= input_das_profit;

                verifiers::common::verify_das_get_change(input_das_profit)?;
            } else {
                debug!("The profit of DAS is not collected completely, so skip counting it.")
            }

            if input_owner_profit > 0 && output_owner_profit == 0 {
                debug!("The profit of owner has been collected.");

                collected = true;
                expected_remain_capacity -= input_owner_profit;

                let owner_lock = util::derive_owner_lock_from_cell(dep_account_cells[0], Source::CellDep)?;
                verifiers::misc::verify_user_get_change(config_main, owner_lock.as_reader(), input_owner_profit)?;
            } else {
                debug!("The profit of owner is not collected completely, so skip counting it.")
            }

            debug!("Verify if the collection is completed properly.");

            das_assert!(
                collected,
                ErrorCode::InvalidTransactionStructure,
                "All profit should be collected at one time, either from DAS or the owner."
            );

            // manual::verify_remain_capacity
            das_assert!(
                output_sub_account_capacity >= expected_remain_capacity - transaction_fee,
                SubAccountCellErrorCode::SubAccountCollectProfitError,
                "The capacity of SubAccountCell in outputs should be at least {}, but only {} found.",
                expected_remain_capacity - transaction_fee,
                output_sub_account_capacity
            );
        }
        Action::CollectSubAccountChannelProfit => {
            let profit_manage_lock = profit_manager_lock();
            let input_profit_manage_cells =
                util::find_cells_by_script(ScriptType::Lock, profit_manage_lock.as_reader().into(), Source::Input)?;

            das_assert!(
                !input_profit_manage_cells.is_empty(),
                SubAccountCellErrorCode::ProfitManagerLockIsRequired,
                "There should be some cells with specific lock in inputs. (expected_lock: {})",
                profit_manage_lock.as_reader()
            );

            verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["das_profit"],
            )?;

            das_assert!(
                input_das_profit > 0,
                SubAccountCellErrorCode::ProfitIsEmpty,
                "The profit of DAS is empty, nothing can be collected."
            );

            das_assert!(
                input_das_profit > output_das_profit,
                SubAccountCellErrorCode::ProfitMustBeCollected,
                "There should be some profit of DAS be collected."
            );
        }
        _ => unreachable!(),
    }

    verify_sub_account_capacity_is_enough(
        config_sub_account,
        input_sub_account_cells[0],
        input_sub_account_capacity,
        &input_sub_account_data,
        output_sub_account_cells[0],
        output_sub_account_capacity,
        &output_sub_account_data,
    )?;

    Ok(())
}

fn verify_sub_account_capacity_is_enough(
    config: ConfigCellSubAccountReader,
    input_index: usize,
    input_capacity: u64,
    input_data: &[u8],
    output_index: usize,
    output_capacity: u64,
    output_data: &[u8],
) -> Result<(), Box<dyn ScriptError>> {
    let basic_capacity = u64::from(config.basic_capacity());
    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();

    das_assert!(
        input_capacity >= input_das_profit + input_owner_profit + basic_capacity,
        SubAccountCellErrorCode::SubAccountCellCapacityError,
        "inputs[{}] The capacity of SubAccountCell should contains profit and basic_capacity, but its not enough.(expected_capacity: {}, current_capacity: {}, das_profit: {}, owner_profit: {})",
        input_index,
        input_das_profit + input_owner_profit + basic_capacity,
        input_capacity,
        input_das_profit,
        input_owner_profit
    );
    das_assert!(
        output_capacity >= output_das_profit + output_owner_profit + basic_capacity,
        SubAccountCellErrorCode::SubAccountCellCapacityError,
        "outputs[{}] The capacity of SubAccountCell should contains profit and basic_capacity, but its not enough.(expected_capacity: {}, current_capacity: {}, das_profit: {}, owner_profit: {})",
        output_index,
        output_das_profit + output_owner_profit + basic_capacity,
        output_capacity,
        output_das_profit,
        output_owner_profit
    );

    Ok(())
}

fn verify_sub_account_transaction_fee(
    config: ConfigCellSubAccountReader,
    input_capacity: u64,
    input_data: &[u8],
    output_capacity: u64,
    output_data: &[u8],
) -> Result<bool, Box<dyn ScriptError>> {
    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();

    let fee = u64::from(config.common_fee());
    let basic_capacity = u64::from(config.basic_capacity());
    let input_remain_fees = input_capacity - input_das_profit - input_owner_profit - basic_capacity;
    let output_remain_fees = output_capacity - output_das_profit - output_owner_profit - basic_capacity;

    das_assert!(
        input_remain_fees <= fee + output_remain_fees,
        ErrorCode::TxFeeSpentError,
        "The transaction fee should be equal to or less than {} .(output_remain_fees: {} = output_capacity - output_profit - basic_capacity, input_remain_fees: {} = ...)",
        fee,
        output_remain_fees,
        input_remain_fees
    );

    Ok(input_remain_fees > output_remain_fees)
}

fn verify_profit_to_das_with_manual(
    cell_index: usize,
    input_data: &[u8],
    output_data: &[u8],
    profit_to_das: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the profit to DAS is recorded properly.");

    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();

    // debug!("input_das_profit: {:?}", input_das_profit);
    // debug!("output_das_profit: {:?}", output_das_profit);

    das_assert!(
        output_das_profit == input_das_profit + profit_to_das,
        SubAccountCellErrorCode::SubAccountProfitError,
        "outputs[{}] The profit of SubAccountCell should contains the new register fees. (input_das_profit: {}, output_das_profit: {}, expected_register_fee: {})",
        cell_index,
        input_das_profit,
        output_das_profit,
        profit_to_das
    );

    Ok(())
}

fn verify_profit_to_das_with_custom_rule(
    _config_sub_account: ConfigCellSubAccountReader,
    input_data: &[u8],
    output_data: &[u8],
    expected_total_profit: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the profit to DAS is calculated properly.");

    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let das_profit = output_das_profit - input_das_profit;

    das_assert!(
        expected_total_profit == das_profit,
        SubAccountCellErrorCode::SubAccountProfitError,
        "The profit to DAS should be calculated properly. (expected_das_profit: {}, das_profit: {})",
        expected_total_profit,
        das_profit
    );

    Ok(())
}
