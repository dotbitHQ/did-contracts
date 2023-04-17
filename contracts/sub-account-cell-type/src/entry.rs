use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::ffi::CStr;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed;
use ckb_std::error::SysError;
use ckb_std::high_level;
use das_core::constants::*;
use das_core::error::*;
use das_core::util::{self, blake2b_256};
use das_core::witness_parser::sub_account::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert as das_assert, code_to_error, data_parser, debug, verifiers, warn};
use das_dynamic_libs::constants::DynLibName;
use das_dynamic_libs::sign_lib::SignLib;
use das_dynamic_libs::{load_2_methods, load_lib, log_loading, new_context};
use das_types::constants::{AccountStatus, DataType, SubAccountAction, SubAccountConfigFlag};
use das_types::packed::*;
use das_types::prelude::{Builder, Entity};
#[cfg(debug_assertions)]
use das_types::prettier::Prettier;
use simple_ast::executor::match_rule_with_account_chars;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running sub-account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );

    match action {
        b"enable_sub_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"recycle_expired_account" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        b"config_sub_account" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

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
            verifiers::misc::verify_no_more_cells_with_same_lock(
                sender_lock.as_reader(),
                &input_account_cells,
                Source::Input,
            )?;

            let input_account_cell_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_cell_reader = input_account_cell_witness.as_reader();
            let output_account_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_cell_reader = output_account_cell_witness.as_reader();

            verifiers::account_cell::verify_status(
                &input_account_cell_reader,
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

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;

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
            let output_sub_account_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
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
                vec!["custom_script", "custom_script_args"],
            )?;

            debug!("Verify if the config fields is updated appropriately ...");

            let sub_account_cell_data = util::load_cell_data(output_sub_account_cells[0], Source::Output)?;
            let flag = data_parser::sub_account_cell::get_flag(&sub_account_cell_data);

            match flag {
                Some(SubAccountConfigFlag::CustomRule) => {
                    verifiers::sub_account_cell::verify_config_is_custom_rule(
                        output_sub_account_cells[0],
                        Source::Output,
                    )?;

                    let sub_account_witness_parser = SubAccountWitnessesParser::new()?;
                    let mut price_rules_empty = true;
                    for data_type in [DataType::SubAccountPriceRule, DataType::SubAccountPreservedRule] {
                        let field = match data_type {
                            DataType::SubAccountPriceRule => String::from("price_rules"),
                            DataType::SubAccountPreservedRule => String::from("preserved_rules"),
                            _ => unreachable!(),
                        };

                        let rules = match sub_account_witness_parser.get_rules(&sub_account_cell_data, data_type) {
                            Ok(rules) => {
                                if data_type == DataType::SubAccountPriceRule {
                                    price_rules_empty = false;
                                }
                                rules
                            }
                            Err(err) => {
                                if err.as_i8() == (ErrorCode::WitnessEmpty as i8) {
                                    continue;
                                } else {
                                    return Err(err);
                                }
                            }
                        };

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

                    das_assert!(
                        !price_rules_empty,
                        SubAccountCellErrorCode::ConfigRulesPriceRulesCanNotEmpty,
                        "The SubAccountCell.witness.price_rules can not be empty."
                    );
                }
                _ => {
                    verifiers::sub_account_cell::verify_config_is_manual(output_sub_account_cells[0], Source::Output)?;
                }
            }
        }
        b"config_sub_account_custom_script" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;

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
            verifiers::misc::verify_no_more_cells_with_same_lock(
                sender_lock.as_reader(),
                &input_account_cells,
                Source::Input,
            )?;

            let input_account_cell_witness =
                util::parse_account_cell_witness(&parser, input_account_cells[0], Source::Input)?;
            let input_account_cell_reader = input_account_cell_witness.as_reader();
            let output_account_cell_witness =
                util::parse_account_cell_witness(&parser, output_account_cells[0], Source::Output)?;
            let output_account_cell_reader = output_account_cell_witness.as_reader();

            verifiers::account_cell::verify_status(
                &input_account_cell_reader,
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

            verifiers::account_cell::verify_account_capacity_not_decrease(
                input_account_cells[0],
                output_account_cells[0],
            )?;

            verifiers::account_cell::verify_account_cell_consistent_with_exception(
                input_account_cells[0],
                output_account_cells[0],
                &input_account_cell_reader,
                &output_account_cell_reader,
                None,
                vec![],
                vec![],
            )?;

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
            let output_sub_account_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

            verify_sub_account_transaction_fee(
                config_sub_account,
                input_sub_account_capacity,
                &input_sub_account_data,
                output_sub_account_capacity,
                &output_sub_account_data,
            )?;

            let input_sub_account_custom_script =
                data_parser::sub_account_cell::get_custom_script(&input_sub_account_data);
            let output_sub_account_custom_script =
                data_parser::sub_account_cell::get_custom_script(&output_sub_account_data);
            let input_sub_account_script_args =
                data_parser::sub_account_cell::get_custom_script_args(&input_sub_account_data);
            let output_sub_account_script_args =
                data_parser::sub_account_cell::get_custom_script_args(&output_sub_account_data);

            // manual::verify_custom_script_changed
            das_assert!(
                input_sub_account_custom_script != output_sub_account_custom_script
                    || input_sub_account_script_args != output_sub_account_script_args,
                SubAccountCellErrorCode::SubAccountCustomScriptError,
                "outputs[{}] The custom script of SubAccountCell should be different in inputs and outputs.",
                output_sub_account_cells[0]
            );

            verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["custom_script", "custom_script_args"],
            )?;
        }
        b"update_sub_account" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_account = parser.configs.account()?;
            let config_sub_account = parser.configs.sub_account()?;

            let timestamp = util::load_oracle_data(OracleCellType::Time)?;
            let quote = util::load_oracle_data(OracleCellType::Quote)?;

            debug!("Verify if the AccountCell in cell_deps has sub-account feature enabled and not expired ...");

            let dep_account_cells = util::find_cells_by_type_id(
                ScriptType::Type,
                config_main.type_id_table().account_cell(),
                Source::CellDep,
            )?;

            verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

            let account_cell_index = dep_account_cells[0];
            let account_cell_source = Source::CellDep;
            let account_cell_witness =
                util::parse_account_cell_witness(&parser, dep_account_cells[0], Source::CellDep)?;
            let account_cell_reader = account_cell_witness.as_reader();
            let account_cell_data = util::load_cell_data(account_cell_index, account_cell_source)?;
            let account_lock = high_level::load_cell_lock(account_cell_index, account_cell_source)?;
            let account_lock_args = account_lock.as_reader().args().raw_data();

            verifiers::account_cell::verify_sub_account_enabled(
                &account_cell_reader,
                account_cell_index,
                account_cell_source,
            )?;

            verifiers::account_cell::verify_account_expiration(
                config_account,
                account_cell_index,
                account_cell_source,
                timestamp,
            )?;

            let mut parent_account = account_cell_reader.account().as_readable();
            parent_account.extend(ACCOUNT_SUFFIX.as_bytes());

            debug!("Verify if the SubAccountCells in inputs and outputs have sufficient capacity and paid transaction fees properly ...");

            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verifiers::common::verify_cell_number_and_position(
                "SubAccountCell",
                &input_sub_account_cells,
                &[0],
                &output_sub_account_cells,
                &[0],
            )?;

            let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

            verify_sub_account_capacity_is_enough(
                config_sub_account,
                input_sub_account_cells[0],
                input_sub_account_capacity,
                &input_sub_account_data,
                output_sub_account_cells[0],
                output_sub_account_capacity,
                &output_sub_account_data,
            )?;

            verify_sub_account_transaction_fee(
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

            debug!("Initialize the dynamic signing libraries ...");

            let mut sign_lib = SignLib::new();
            // ⚠️ This must be present at the top level, as we will need to use the libraries later.

            // let mut ckb_context = new_context!();
            // log_loading!(DynLibName::CKBSignhash, config_main.das_lock_type_id_table());
            // let ckb_lib = load_lib!(ckb_context, DynLibName::CKBSignhash, config_main.das_lock_type_id_table());
            // sign_lib.ckb_signhash = load_2_methods!(ckb_lib);

            if cfg!(not(feature = "dev")) {
                let mut eth_context = new_context!();
                log_loading!(DynLibName::ETH, config_main.das_lock_type_id_table());
                let eth_lib = load_lib!(eth_context, DynLibName::ETH, config_main.das_lock_type_id_table());
                sign_lib.eth = load_2_methods!(eth_lib);

                let mut tron_context = new_context!();
                log_loading!(DynLibName::TRON, config_main.das_lock_type_id_table());
                let tron_lib = load_lib!(tron_context, DynLibName::TRON, config_main.das_lock_type_id_table());
                sign_lib.tron = load_2_methods!(tron_lib);

                let mut doge_context = new_context!();
                log_loading!(DynLibName::DOGE, config_main.das_lock_type_id_table());
                let doge_lib = load_lib!(doge_context, DynLibName::DOGE, config_main.das_lock_type_id_table());
                sign_lib.doge = load_2_methods!(doge_lib);
            }

            // Initiate some variables used again and again in the following codes.
            let sub_account_parser = SubAccountWitnessesParser::new()?;
            let flag = data_parser::sub_account_cell::get_flag(&output_sub_account_data);
            let parent_expired_at = data_parser::account_cell::get_expired_at(&account_cell_data);
            let header = util::load_header(input_sub_account_cells[0], Source::Input)?;
            let sub_account_last_updated_at = util::get_timestamp_from_header(header.as_reader());

            let mut manual_mint_list_smt_root = None;

            let mut custom_script_params = Vec::new();
            let mut custom_script_type_id = None;

            let mut custom_price_rules = None;
            let mut custom_preserved_rules = None;

            let all_inputs_balance_cells = util::find_all_balance_cells(config_main, Source::Input)?;
            if sub_account_parser.contains_creation {
                debug!("Found `create` action in this transaction, do some common verfications ...");

                match sub_account_parser.get_mint_sign(account_lock_args) {
                    Some(Ok(witness)) => {
                        debug!("The SubAccountMintSignWitness is exist, verifying the signature for manual mint ...",);

                        verifiers::sub_account_cell::verify_sub_account_mint_sign_not_expired(
                            &sub_account_parser,
                            &witness,
                            parent_expired_at,
                            sub_account_last_updated_at,
                        )?;
                        verifiers::sub_account_cell::verify_sub_account_mint_sign(&witness, &sign_lib)?;

                        let mut tmp = [0u8; 32];
                        tmp.copy_from_slice(&witness.account_list_smt_root);
                        manual_mint_list_smt_root = Some(tmp);
                    }
                    Some(Err(err)) => {
                        return Err(err);
                    }
                    None => {}
                }

                match flag {
                    Some(SubAccountConfigFlag::CustomScript) => {
                        debug!(
                            "The SubAccountCell.data.flag is {}, skip prepare the custom script params ...",
                            SubAccountConfigFlag::CustomScript
                        );

                        debug!("Push action into custom_script_params.");

                        // TODO This can be removed now, but we need update the custom script first.
                        let action_str = String::from_utf8(action.to_vec()).unwrap();
                        custom_script_params.push(action_str);

                        debug!("Try to find the QuoteCell from cell_deps and push quote into custom_script_params.");

                        let quote = util::load_oracle_data(OracleCellType::Quote)?;
                        custom_script_params.push(util::hex_string(&quote.to_le_bytes()));

                        let input_das_profit =
                            data_parser::sub_account_cell::get_das_profit(&input_sub_account_data).unwrap();
                        let output_das_profit =
                            data_parser::sub_account_cell::get_das_profit(&output_sub_account_data).unwrap();
                        let input_owner_profit =
                            data_parser::sub_account_cell::get_owner_profit(&input_sub_account_data).unwrap();
                        let output_owner_profit =
                            data_parser::sub_account_cell::get_owner_profit(&output_sub_account_data).unwrap();
                        let owner_profit = output_owner_profit - input_owner_profit;
                        let das_profit = output_das_profit - input_das_profit;

                        custom_script_params.push(util::hex_string(&owner_profit.to_le_bytes()));
                        custom_script_params.push(util::hex_string(&das_profit.to_le_bytes()));

                        let custom_script_args =
                            data_parser::sub_account_cell::get_custom_script_args(&input_sub_account_data).unwrap();
                        custom_script_params.push(util::hex_string(custom_script_args));

                        let args = match data_parser::sub_account_cell::get_custom_script(&input_sub_account_data) {
                            Some(val) => val,
                            None => {
                                return Err(code_to_error!(SubAccountCellErrorCode::SubAccountCustomScriptEmpty));
                            }
                        };
                        let type_of_custom_script = Script::new_builder()
                            .code_hash(Hash::from(TYPE_ID_CODE_HASH))
                            .hash_type(Byte::from(1))
                            .args(Bytes::from(args))
                            .build();
                        let type_id = util::blake2b_256(type_of_custom_script.as_slice());

                        debug!("The type ID of custom script is: 0x{}", util::hex_string(&type_id));

                        custom_script_type_id = Some(type_id);

                        // CAREFUL This is very important, only update it with fully understanding the requirements.
                        debug!("Verify if there is no BalanceCells of the parent AccountCell's owner is spent.");

                        verifiers::common::verify_cell_number("BalanceCell", &all_inputs_balance_cells, 0, &[], 0)?;
                    }
                    Some(SubAccountConfigFlag::CustomRule) => {
                        debug!(
                            "The SubAccountCell.data.flag is {}, pasing custom rules ...",
                            SubAccountConfigFlag::CustomRule
                        );

                        custom_price_rules =
                            Some(sub_account_parser.get_rules(&input_sub_account_data, DataType::SubAccountPriceRule)?);
                        custom_preserved_rules = match sub_account_parser
                            .get_rules(&input_sub_account_data, DataType::SubAccountPreservedRule)
                        {
                            Ok(rules) => Some(rules),
                            Err(err) => {
                                if err.as_i8() == (ErrorCode::WitnessEmpty as i8) {
                                    None
                                } else {
                                    return Err(err);
                                }
                            }
                        };
                    }
                    _ => {
                        debug!("The SubAccountCell.data.flag is {} ...", SubAccountConfigFlag::Manual);
                    }
                }

                verifiers::account_cell::verify_status(
                    &account_cell_reader,
                    AccountStatus::Normal,
                    account_cell_index,
                    account_cell_source,
                )?;

                verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                    input_sub_account_cells[0],
                    output_sub_account_cells[0],
                    vec!["smt_root", "das_profit", "owner_profit"],
                )?;
            } else {
                // CAREFUL This is very important, only update it with fully understanding the requirements.
                debug!("Verify if there is no BalanceCells are spent.");

                verifiers::common::verify_cell_number("BalanceCell", &all_inputs_balance_cells, 0, &[], 0)?;
            }

            // 以前这里好像是为了安全或者便于计算才限制了只能使用 owner 的 cell
            // das_assert!(
            //     &input_balance_cells == &all_inputs_balance_cells,
            //     ErrorCode::BalanceCellCanNotBeSpent,
            //     "Only BalanceCells which belong to the parent AccountCell's owner can be spent in this transaction."
            // );

            // das_assert!(false, ErrorCode::HardCodedError, "");

            if sub_account_parser.contains_edition {
                debug!("Found `edit` action in this transaction, do some common verfications ...");

                if !sub_account_parser.contains_creation {
                    debug!("There is no `create` action found, verify the SubAccountCell is consistent in inputs and outputs.");

                    verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                        input_sub_account_cells[0],
                        output_sub_account_cells[0],
                        vec!["smt_root"],
                    )?;
                }
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

            let mut profit_to_das_manual_mint = 0;
            let mut profit_to_das = 0;
            let mut profit_total = 0;
            for (i, witness_ret) in sub_account_parser.iter().enumerate() {
                if let Err(e) = witness_ret {
                    return Err(e);
                }

                let witness = witness_ret.unwrap();
                let sub_account_reader = witness.sub_account.as_reader();

                verifiers::sub_account_cell::verify_suffix_with_parent_account(
                    witness.index,
                    sub_account_reader,
                    &parent_account,
                )?;

                match witness.action {
                    SubAccountAction::Create => {
                        smt_verify_sub_account_is_creatable(&prev_root, &witness)?;

                        debug!(
                            "  witnesses[{:>2}] Verify if the account is registrable.",
                            witness.index
                        );

                        let account_chars = witness.sub_account.account();
                        let account_chars_reader = account_chars.as_reader();
                        let mut account_bytes = account_chars.as_readable();
                        account_bytes.extend(sub_account_reader.suffix().raw_data());
                        let account = String::from_utf8(account_bytes)
                            .map_err(|_| code_to_error!(SubAccountCellErrorCode::BytesToStringFailed))?;

                        verifiers::account_cell::verify_account_chars(&parser, account_chars_reader)?;
                        verifiers::account_cell::verify_account_chars_min_length(account_chars_reader)?;
                        verifiers::account_cell::verify_account_chars_max_length(&parser, account_chars_reader)?;

                        verifiers::sub_account_cell::verify_initial_properties(
                            witness.index,
                            sub_account_reader,
                            timestamp,
                        )?;

                        let expired_at = u64::from(sub_account_reader.expired_at());
                        let registered_at = u64::from(sub_account_reader.registered_at());
                        let expiration_years = (expired_at - registered_at) / YEAR_SEC;

                        let mut manually_minted = false;
                        if let Some(root) = manual_mint_list_smt_root.clone() {
                            match smt_verify_sub_account_is_in_signed_list(root, &witness) {
                                Ok(()) => {
                                    debug!(
                                        "  witnesses[{:>2}] The account is in the signed mint list, it can be register without payment.",
                                        witness.index
                                    );
                                    manually_minted = true;
                                    profit_to_das_manual_mint +=
                                        u64::from(config_sub_account.new_sub_account_price()) * expiration_years;
                                }
                                Err(_) => {}
                            }
                        }

                        if !manually_minted {
                            debug!(
                                "  witnesses[{:>2}] The account is not in the signed mint list, calculating its price ...",
                                witness.index
                            );

                            match flag {
                                Some(SubAccountConfigFlag::CustomScript) => {
                                    debug!(
                                        "  witnesses[{:>2}] The SubAccountCell.data.flag is {}, record registered years and pass to custom scripts later ...",
                                        witness.index,
                                        SubAccountConfigFlag::CustomScript
                                    );

                                    let mut custom_script_param = expiration_years.to_le_bytes().to_vec();
                                    custom_script_param.append(&mut sub_account_reader.account().as_slice().to_vec());
                                    custom_script_params.push(util::hex_string(&custom_script_param));

                                    // This variable will be treat as the minimal profit to DAS no matter the custom script exist or not.
                                    profit_to_das +=
                                        u64::from(config_sub_account.new_sub_account_price()) * expiration_years;
                                }
                                Some(SubAccountConfigFlag::CustomRule) => {
                                    debug!(
                                        "  witnesses[{:>2}] The SubAccountCell.data.flag is {}, execute the custome rules to check if the account is preserved and calculate its price ...",
                                        witness.index,
                                        SubAccountConfigFlag::CustomRule
                                    );

                                    if let Some(rules) = custom_preserved_rules.as_ref() {
                                        if let Some(rule) = match_rule_with_account_chars(&rules, account_chars_reader, &account)
                                            .map_err(|_| {
                                                code_to_error!(SubAccountCellErrorCode::ConfigRulesHasSyntaxError)
                                            })?
                                        {
                                            warn!(
                                                "  witnesses[{:>2}] The new SubAccount should be preserved.(matched rule: {})",
                                                witness.index,
                                                rule.index
                                            );
                                            return Err(code_to_error!(SubAccountCellErrorCode::AccountIsPreserved));
                                        }
                                    }

                                    if let Some(rules) = custom_price_rules.as_ref() {
                                        if let Some(rule) =
                                            match_rule_with_account_chars(&rules, account_chars_reader, &account)
                                                .map_err(|_| {
                                                    code_to_error!(SubAccountCellErrorCode::ConfigRulesHasSyntaxError)
                                                })?
                                        {
                                            let profit = util::calc_yearly_capacity(rule.price, quote, 0) * expiration_years;
                                            profit_total += profit;
                                            profit_to_das += profit * u32::from(config_sub_account.new_sub_account_custom_price_das_profit_rate()) as u64 / RATE_BASE;;

                                            debug!(
                                                "  witnesses[{:>2}] account: {}, matched rule: {}, profit: {} in shannon",
                                                witness.index, account, rule.index, profit
                                            );
                                        } else {
                                            warn!(
                                                "  witnesses[{:>2}] The account {} has no price, it is can not be registered.",
                                                witness.index,
                                                account
                                            );
                                            return Err(code_to_error!(SubAccountCellErrorCode::AccountHasNoPrice));
                                        }
                                    }
                                }
                                _ => {
                                    warn!(
                                        "  witnesses[{:>2}] The new SubAccount should be in the signed mint list.",
                                        witness.index
                                    );
                                    return Err(code_to_error!(SubAccountCellErrorCode::CanNotMint));
                                }
                            }
                        }
                    }
                    SubAccountAction::Edit => {
                        let new_sub_account =
                            generate_new_sub_account_by_edit_value(witness.sub_account.clone(), &witness.edit_value)?;
                        let new_sub_account_reader = new_sub_account.as_reader();

                        debug!(
                            "witnesses[{}] Calculated new sub-account structure is: {}",
                            witness.index,
                            new_sub_account_reader.as_prettier()
                        );

                        smt_verify_sub_account_is_editable(&prev_root, &witness, new_sub_account_reader)?;

                        verifiers::sub_account_cell::verify_unlock_role(&witness)?;
                        verifiers::sub_account_cell::verify_sub_account_edit_sign_not_expired(
                            &witness,
                            parent_expired_at,
                            sub_account_last_updated_at,
                        )?;
                        verifiers::sub_account_cell::verify_sub_account_edit_sign(&witness, &sign_lib)?;
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

                        match &witness.edit_value {
                            SubAccountEditValue::Owner(new_args) | SubAccountEditValue::Manager(new_args) => {
                                let sub_account_reader = witness.sub_account.as_reader();
                                let current_args = sub_account_reader.lock().args().raw_data();
                                let (
                                    current_owner_type,
                                    current_owner_args,
                                    current_manager_type,
                                    current_manager_args,
                                ) = data_parser::das_lock_args::get_owner_and_manager(current_args)?;
                                let (new_owner_type, new_owner_args, new_manager_type, new_manager_args) =
                                    data_parser::das_lock_args::get_owner_and_manager(new_args)?;

                                if let SubAccountEditValue::Owner(_) = &witness.edit_value {
                                    debug!(
                                        "witnesses[{}] Verify if owner has been changed correctly.",
                                        witness.index
                                    );

                                    das_assert!(
                                        current_owner_type != new_owner_type || current_owner_args != new_owner_args,
                                        SubAccountCellErrorCode::SubAccountEditLockError,
                                        "witnesses[{}] The owner fields in args should be consistent.",
                                        witness.index
                                    );

                                    // Skip verifying manger, because owner has been changed.
                                } else {
                                    debug!(
                                        "witnesses[{}] Verify if manager has been changed correctly.",
                                        witness.index
                                    );

                                    das_assert!(
                                        current_owner_type == new_owner_type && current_owner_args == new_owner_args,
                                        SubAccountCellErrorCode::SubAccountEditLockError,
                                        "witnesses[{}] The owner fields in args should be consistent.",
                                        witness.index
                                    );

                                    das_assert!(
                                        current_manager_type != new_manager_type
                                            || current_manager_args != new_manager_args,
                                        SubAccountCellErrorCode::SubAccountEditLockError,
                                        "witnesses[{}] The manager fields in args should be changed.",
                                        witness.index
                                    );
                                }
                            }
                            SubAccountEditValue::Records(records) => {
                                verifiers::account_cell::verify_records_keys(&parser, records.as_reader())?;
                            }
                            // manual::verify_expired_at_not_editable
                            SubAccountEditValue::ExpiredAt(_) => {
                                warn!(
                                    "witnesses[{}] Can not edit witness.sub_account.expired_at in this transaction.",
                                    witness.index
                                );
                                return Err(code_to_error!(SubAccountCellErrorCode::SubAccountFieldNotEditable));
                            }
                            // manual::verify_edit_value_not_empty
                            SubAccountEditValue::None => {
                                warn!(
                                    "witnesses[{}] The witness.edit_value should not be empty.",
                                    witness.index
                                );
                                return Err(code_to_error!(SubAccountCellErrorCode::SubAccountFieldNotEditable));
                            }
                        }
                    }
                    SubAccountAction::Renew => todo!(),
                    SubAccountAction::Recycle => todo!(),
                }

                prev_root = witness.new_root.clone();

                if i == sub_account_parser.len() - 1 {
                    debug!(
                        "witnesses[{}] Verify if the last witness.new_root is consistent with the latest SMT root in the SubAccountCell in the outputs..",
                        witness.index
                    );

                    let latest_root_in_witness = witness.new_root.as_slice();
                    das_assert!(
                        latest_root_in_witness == latest_root,
                        SubAccountCellErrorCode::SubAccountWitnessMismatched,
                        "The latest SMT root in witnesses should be consistent with the latest SMT root in the SubAccountCell in the outputs.(in_witness: {}, in_data: {})",
                        util::hex_string(latest_root_in_witness),
                        util::hex_string(&latest_root)
                    );
                }
            }

            if sub_account_parser.contains_creation {
                debug!("Verify if the profit distribution is correct.");

                match flag {
                    Some(SubAccountConfigFlag::CustomScript) => {
                        verify_there_is_only_one_lock_for_normal_cells(
                            input_sub_account_cells[0],
                            output_sub_account_cells[0],
                        )?;

                        verify_profit_to_das_with_custom_script(
                            config_sub_account,
                            profit_to_das + profit_to_das_manual_mint,
                            &input_sub_account_data,
                            &output_sub_account_data,
                        )?;

                        let type_id = custom_script_type_id.unwrap();

                        debug!("Execute custom script by type ID: 0x{}", util::hex_string(&type_id));

                        let params_with_nul = custom_script_params
                            .iter()
                            .map(|val| {
                                let mut ret = val.as_bytes().to_vec();
                                ret.push(0);
                                ret
                            })
                            .collect::<Vec<_>>();
                        let mut total_size = 0;
                        let params = params_with_nul
                            .iter()
                            .map(|val| unsafe {
                                total_size += val.len();
                                CStr::from_bytes_with_nul_unchecked(val.as_slice())
                            })
                            .collect::<Vec<_>>();

                        debug!("The total size of custom script params: {} bytes", total_size);

                        high_level::exec_cell(&type_id, ScriptHashType::Type, 0, 0, &params)
                            .map_err(Error::<ErrorCode>::from)?;
                    }
                    Some(SubAccountConfigFlag::CustomRule) => {
                        verify_profit_to_das_and_owner(
                            config_sub_account,
                            &input_sub_account_data,
                            &output_sub_account_data,
                            profit_to_das + profit_to_das_manual_mint,
                            profit_total - (profit_to_das + profit_to_das_manual_mint),
                        )?;
                    }
                    _ => {
                        verify_profit_to_das(
                            output_sub_account_cells[0],
                            &input_sub_account_data,
                            &output_sub_account_data,
                            profit_to_das + profit_to_das_manual_mint,
                        )?;
                    }
                }
            }
        }
        b"collect_sub_account_profit" => {
            parser.parse_cell()?;
            let config_main = parser.configs.main()?;
            let config_sub_account = parser.configs.sub_account()?;

            debug!("Try to find the SubAccountCells from cell_deps ...");

            let (input_sub_account_cells, output_sub_account_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verifiers::common::verify_cell_number_and_position(
                "SubAccountCell",
                &input_sub_account_cells,
                &[0],
                &output_sub_account_cells,
                &[0],
            )?;

            let input_sub_account_capacity = high_level::load_cell_capacity(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_capacity =
                high_level::load_cell_capacity(output_sub_account_cells[0], Source::Output)?;
            let input_sub_account_data = high_level::load_cell_data(input_sub_account_cells[0], Source::Input)?;
            let output_sub_account_data = high_level::load_cell_data(output_sub_account_cells[0], Source::Output)?;

            verify_sub_account_capacity_is_enough(
                config_sub_account,
                input_sub_account_cells[0],
                input_sub_account_capacity,
                &input_sub_account_data,
                output_sub_account_cells[0],
                output_sub_account_capacity,
                &output_sub_account_data,
            )?;

            verifiers::sub_account_cell::verify_sub_account_cell_is_consistent(
                input_sub_account_cells[0],
                output_sub_account_cells[0],
                vec!["das_profit", "owner_profit"],
            )?;

            debug!("Try to find the AccountCell from cell_deps ...");

            let dep_account_cells = util::find_cells_by_type_id(
                ScriptType::Type,
                config_main.type_id_table().account_cell(),
                Source::CellDep,
            )?;

            verifiers::common::verify_cell_dep_number("AccountCell", &dep_account_cells, 1)?;

            let account_cell_witness =
                util::parse_account_cell_witness(&parser, dep_account_cells[0], Source::CellDep)?;
            let account_cell_reader = account_cell_witness.as_reader();

            verifiers::sub_account_cell::verify_sub_account_parent_id(
                input_sub_account_cells[0],
                Source::Input,
                account_cell_reader.id().raw_data(),
            )?;

            let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_sub_account_data).unwrap();
            let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_sub_account_data).unwrap();
            let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_sub_account_data).unwrap();
            let output_owner_profit =
                data_parser::sub_account_cell::get_owner_profit(&output_sub_account_data).unwrap();

            das_assert!(
                input_das_profit != 0 || input_owner_profit != 0,
                ErrorCode::InvalidTransactionStructure,
                "Either the profit of DAS or the profit of owner should not be 0 ."
            );

            let mut collected = false;
            let mut expected_remain_capacity = input_sub_account_capacity;
            let transaction_fee = u64::from(config_sub_account.common_fee());

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
        b"confirm_expired_account_auction" => {
            util::require_type_script(
                &parser,
                TypeScript::AccountCellType,
                Source::Input,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

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
) -> Result<(), Box<dyn ScriptError>> {
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

    Ok(())
}

fn verify_profit_to_das(
    cell_index: usize,
    input_data: &[u8],
    output_data: &[u8],
    profit_to_das: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the profit to DAS is recorded properly.");

    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();

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

fn verify_profit_to_das_with_custom_script(
    config_sub_account: ConfigCellSubAccountReader,
    minimal_profit_to_das: u64,
    input_data: &[u8],
    output_data: &[u8],
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the profit to DAS is calculated from rate of config properly.");

    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();
    let owner_profit = output_owner_profit - input_owner_profit;
    let das_profit = output_das_profit - input_das_profit;
    let total_profit = owner_profit + das_profit;
    let profit_rate = u32::from(config_sub_account.new_sub_account_custom_price_das_profit_rate());

    das_assert!(
        das_profit >= minimal_profit_to_das,
        SubAccountCellErrorCode::SubAccountProfitError,
        "The profit to DAS should be greater than or equal to the minimal profit which is 1 CKB per account. (das_profit: {}, minimal_profit_to_das: {})",
        das_profit,
        minimal_profit_to_das
    );

    // CAREFUL: Overflow risk
    let mut expected_das_profit = total_profit * profit_rate as u64 / RATE_BASE;
    if expected_das_profit < minimal_profit_to_das {
        expected_das_profit = minimal_profit_to_das;
    }

    das_assert!(
        expected_das_profit == das_profit,
        SubAccountCellErrorCode::SubAccountProfitError,
        "The profit to DAS should be calculated from rate of config properly. (expected_das_profit: {}, das_profit: {})",
        expected_das_profit,
        das_profit
    );

    Ok(())
}

fn verify_profit_to_das_and_owner(
    config_sub_account: ConfigCellSubAccountReader,
    input_data: &[u8],
    output_data: &[u8],
    minimal_profit_to_das: u64,
    expected_profit_to_owner: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify the profit to DAS is calculated from rate of config properly.");

    let input_das_profit = data_parser::sub_account_cell::get_das_profit(&input_data).unwrap();
    let output_das_profit = data_parser::sub_account_cell::get_das_profit(&output_data).unwrap();
    let input_owner_profit = data_parser::sub_account_cell::get_owner_profit(&input_data).unwrap();
    let output_owner_profit = data_parser::sub_account_cell::get_owner_profit(&output_data).unwrap();
    let owner_profit = output_owner_profit - input_owner_profit;
    let das_profit = output_das_profit - input_das_profit;
    let total_profit = owner_profit + das_profit;
    let profit_rate = u32::from(config_sub_account.new_sub_account_custom_price_das_profit_rate());

    das_assert!(
        das_profit >= minimal_profit_to_das,
        SubAccountCellErrorCode::SubAccountProfitError,
        "The profit to DAS should be greater than or equal to the minimal profit which is 1 CKB per account. (das_profit: {}, minimal_profit_to_das: {})",
        das_profit,
        minimal_profit_to_das
    );

    // CAREFUL: Overflow risk
    let mut expected_das_profit = total_profit * profit_rate as u64 / RATE_BASE;
    if expected_das_profit < minimal_profit_to_das {
        expected_das_profit = minimal_profit_to_das;
    }

    das_assert!(
        expected_das_profit == das_profit,
        SubAccountCellErrorCode::SubAccountProfitError,
        "The profit to DAS should be calculated from rate of config properly. (expected_das_profit: {}, das_profit: {})",
        expected_das_profit,
        das_profit
    );

    das_assert!(
        expected_profit_to_owner == owner_profit,
        SubAccountCellErrorCode::SubAccountProfitError,
        "The profit to owner should be exactly follow the price rules. (expected_owner_profit: {}, owner_profit: {})",
        expected_profit_to_owner,
        owner_profit
    );

    Ok(())
}

fn verify_there_is_only_one_lock_for_normal_cells(
    input_sub_account_cell: usize,
    output_sub_account_cell: usize,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify there is only one lock for cells which is not SubAccountCell.");

    let mut lock_hash = None;
    for (field_name, source) in [("inputs", Source::Input), ("outputs", Source::Output)] {
        let mut i = 0;
        loop {
            if source == Source::Input && i == input_sub_account_cell {
                i += 1;
                continue;
            } else if source == Source::Output && i == output_sub_account_cell {
                i += 1;
                continue;
            }

            let ret = high_level::load_cell_lock_hash(i, source);
            match ret {
                Ok(val) => {
                    if lock_hash.is_none() {
                        lock_hash = Some(val);
                    } else {
                        das_assert!(
                            lock_hash == Some(val),
                            SubAccountCellErrorCode::SubAccountNormalCellLockLimit,
                            "{}[{}] There should be only one lock for cells which is not SubAccountCell.",
                            field_name,
                            i
                        );
                    }
                }
                Err(SysError::IndexOutOfBound) => {
                    break;
                }
                Err(err) => {
                    return Err(Error::<ErrorCode>::from(err).into());
                }
            }

            i += 1;
        }
    }

    Ok(())
}

fn gen_smt_key_by_account_id(account_id: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let key_pre = [account_id, &[0u8; 12]].concat();
    key.copy_from_slice(&key_pre);
    key
}

fn smt_verify_sub_account_is_in_signed_list(
    root: [u8; 32],
    witness: &SubAccountWitness,
) -> Result<(), Box<dyn ScriptError>> {
    let proof = &witness.edit_value_bytes;
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let value = util::blake2b_256(witness.sub_account.lock().args().as_reader().raw_data());

    debug!(
        "  witnesses[{:>2}] Verify if {} is exist in the SubAccountMintSignWitness.account_list_smt_root.(key: 0x{})",
        witness.index,
        witness.sub_account.account().as_prettier(),
        util::hex_string(&key)
    );

    verifiers::common::verify_smt_proof(key, value, root, proof)?;

    Ok(())
}

fn smt_verify_sub_account_is_creatable(
    prev_root: &[u8],
    witness: &SubAccountWitness,
) -> Result<(), Box<dyn ScriptError>> {
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!(
        "  witnesses[{:>2}] Verify if the sub-account was not exist in the SMT before.(key: 0x{})",
        witness.index,
        util::hex_string(&key)
    );
    let zero_val = [0u8; 32];
    verifiers::common::verify_smt_proof(key, zero_val, prev_root.try_into().unwrap(), proof)?;

    debug!(
        "  witnesses[{:>2}] Verify if the sub-account is in the SMT now.",
        witness.index
    );
    let current_root = witness.new_root.as_slice();
    let current_val = blake2b_256(witness.sub_account.as_slice()).to_vec().try_into().unwrap();
    verifiers::common::verify_smt_proof(key, current_val, current_root.try_into().unwrap(), proof)?;

    Ok(())
}

fn smt_verify_sub_account_is_editable(
    prev_root: &[u8],
    witness: &SubAccountWitness,
    new_sub_account: SubAccountReader,
) -> Result<(), Box<dyn ScriptError>> {
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!(
        "  witnesses[{:>2}] Verify if the current state of the sub-account was in the SMT before.(key: 0x{})",
        witness.index,
        util::hex_string(&key)
    );
    let prev_val: [u8; 32] = blake2b_256(witness.sub_account.as_slice()).to_vec().try_into().unwrap();
    // debug!("prev_val = 0x{}", util::hex_string(&prev_val));
    // debug!("prev_val_raw = 0x{}", util::hex_string(witness.sub_account.as_slice()));
    // debug!("prev_val_prettier = {}", witness.sub_account.as_prettier());
    verifiers::common::verify_smt_proof(key, prev_val, prev_root.try_into().unwrap(), proof)?;

    debug!(
        "  witnesses[{:>2}] Verify if the new state of the sub-account is in the SMT now.",
        witness.index
    );
    let current_root = witness.new_root.as_slice();
    let current_val: [u8; 32] = blake2b_256(new_sub_account.as_slice()).to_vec().try_into().unwrap();
    // debug!("current_val = 0x{}", util::hex_string(&current_val));
    // debug!("current_val_raw = 0x{}", util::hex_string(new_sub_account.as_slice()));
    // debug!("current_val_prettier = {}", new_sub_account.as_prettier());
    verifiers::common::verify_smt_proof(key, current_val, current_root.try_into().unwrap(), proof)?;

    Ok(())
}

fn generate_new_sub_account_by_edit_value(
    sub_account: SubAccount,
    edit_value: &SubAccountEditValue,
) -> Result<SubAccount, Box<dyn ScriptError>> {
    let current_nonce = u64::from(sub_account.nonce());

    let mut sub_account_builder = match edit_value {
        SubAccountEditValue::ExpiredAt(val) => {
            let sub_account_builder = sub_account.as_builder();
            sub_account_builder.expired_at(val.to_owned())
        }
        SubAccountEditValue::Owner(val) | SubAccountEditValue::Manager(val) => {
            let mut lock_builder = sub_account.lock().as_builder();
            let mut sub_account_builder = sub_account.as_builder();

            // Verify if the edit_value is a valid format.
            data_parser::das_lock_args::get_owner_and_manager(val)?;

            lock_builder = lock_builder.args(Bytes::from(val.to_owned()));
            sub_account_builder = sub_account_builder.lock(lock_builder.build());

            if let SubAccountEditValue::Owner(_) = edit_value {
                sub_account_builder = sub_account_builder.records(Records::default())
            }

            sub_account_builder
        }
        SubAccountEditValue::Records(val) => {
            let sub_account_builder = sub_account.as_builder();
            sub_account_builder.records(val.to_owned())
        }
        _ => unreachable!(),
    };

    // Every time a sub-account is edited, its nonce must  increase by 1 .
    sub_account_builder = sub_account_builder.nonce(Uint64::from(current_nonce + 1));

    Ok(sub_account_builder.build())
}
