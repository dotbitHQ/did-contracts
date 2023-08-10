use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use das_core::constants::*;
use das_core::error::{ErrorCode, ScriptError, SubAccountCellErrorCode};
use das_core::util::{self, blake2b_256};
use das_core::witness_parser::sub_account::{SubAccountEditValue, SubAccountWitness, SubAccountWitnessesParser};
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, das_assert, data_parser, debug, verifiers, warn};
use das_dynamic_libs::sign_lib::SignLib;
use das_types::constants::*;
use das_types::packed::*;
use das_types::prelude::{Builder, Entity};
#[cfg(debug_assertions)]
use das_types::prettier::Prettier;
use simple_ast::executor::match_rule_with_account_chars;
use simple_ast::types as ast_types;

pub struct SubAction<'a> {
    sign_lib: SignLib,

    timestamp: u64,
    quote: u64,
    flag: SubAccountConfigFlag,
    custom_rule_flag: SubAccountCustomRuleFlag,
    sub_account_last_updated_at: u64,

    parser: &'a WitnessesParser,
    config_account: ConfigCellAccountReader<'a>,
    config_sub_account: ConfigCellSubAccountReader<'a>,
    parent_account: &'a [u8],
    parent_expired_at: u64,

    // profit fields
    pub minimal_required_das_profit: u64,
    pub profit_total: u64,
    pub profit_from_manual_mint: u64,
    pub profit_from_manual_renew: u64,
    pub profit_from_manual_renew_by_other: u64,

    // manual mint fields
    manual_mint_list_smt_root: &'a Option<[u8; 32]>,
    manual_renew_list_smt_root: &'a Option<[u8; 32]>,

    // custom script fields
    pub custom_script_params: Vec<String>,

    // custom rule fields
    custom_preserved_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
    custom_price_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
}

impl<'a> SubAction<'a> {
    pub fn new(
        sign_lib: SignLib,
        timestamp: u64,
        quote: u64,
        flag: SubAccountConfigFlag,
        custom_rule_flag: SubAccountCustomRuleFlag,
        sub_account_last_updated_at: u64,
        parser: &'a WitnessesParser,
        config_account: ConfigCellAccountReader<'a>,
        config_sub_account: ConfigCellSubAccountReader<'a>,
        parent_account: &'a [u8],
        parent_expired_at: u64,
        manual_mint_list_smt_root: &'a Option<[u8; 32]>,
        manual_renew_list_smt_root: &'a Option<[u8; 32]>,
        custom_script_params: Vec<String>,
        custom_preserved_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
        custom_price_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
    ) -> Self {
        Self {
            sign_lib,
            timestamp,
            quote,
            flag,
            custom_rule_flag,
            sub_account_last_updated_at,
            parser,
            config_account,
            config_sub_account,
            parent_account,
            parent_expired_at,
            minimal_required_das_profit: 0,
            profit_total: 0,
            profit_from_manual_mint: 0,
            profit_from_manual_renew: 0,
            profit_from_manual_renew_by_other: 0,
            manual_mint_list_smt_root,
            manual_renew_list_smt_root,
            custom_script_params,
            custom_preserved_rules,
            custom_price_rules,
        }
    }

    pub fn dispatch(
        &mut self,
        witness: &SubAccountWitness,
        prev_root: &[u8],
        witness_parser: &SubAccountWitnessesParser,
    ) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();

        verifiers::sub_account_cell::verify_suffix_with_parent_account(
            witness.index,
            sub_account_reader,
            self.parent_account,
        )?;

        match witness.action {
            SubAccountAction::Create => self.create(witness, prev_root)?,
            SubAccountAction::Renew => self.renew(witness, prev_root)?,
            SubAccountAction::Edit => self.edit(witness, prev_root, witness_parser)?,
            SubAccountAction::Recycle => self.recycle(witness, prev_root)?,
        }

        Ok(())
    }

    fn create(&mut self, witness: &SubAccountWitness, prev_root: &[u8]) -> Result<(), Box<dyn ScriptError>> {
        smt_verify_sub_account_is_creatable(&prev_root, &witness)?;

        debug!(
            "  witnesses[{:>2}] Verify if the account is registrable.",
            witness.index
        );

        let sub_account_reader = witness.sub_account.as_reader();
        let (account, account_chars_reader) = gen_account_from_witness(sub_account_reader)?;

        verifiers::account_cell::verify_account_chars(self.parser, account_chars_reader)?;
        verifiers::account_cell::verify_account_chars_min_length(account_chars_reader)?;
        verifiers::account_cell::verify_account_chars_max_length(self.parser, account_chars_reader)?;

        verifiers::sub_account_cell::verify_initial_properties(
            self.parser,
            witness.index,
            sub_account_reader,
            self.timestamp,
        )?;

        // The verifiers::sub_account_cell::verify_initial_properties has ensured the expiration_years is >= 1 year.
        let expired_at = u64::from(sub_account_reader.expired_at());
        let registered_at = u64::from(sub_account_reader.registered_at());
        let expiration_years = (expired_at - registered_at) / YEAR_SEC;
        let expiration_tolerance = (expired_at - registered_at) % YEAR_SEC;

        debug!(
            "  witnesses[{:>2}] The account is registered for {} years.",
            witness.index, expiration_years
        );

        das_assert!(
            expiration_years >= 1,
            SubAccountCellErrorCode::ExpirationYearsTooShort,
            "  witnesses[{:>2}] The expired_at date should be more than 1 year after the registered_at date.",
            witness.index
        );

        das_assert!(
            expiration_tolerance <= DAY_SEC,
            SubAccountCellErrorCode::ExpirationToleranceReached,
            "  witnesses[{:>2}] The expired_at date reached maximum tolerance.",
            witness.index
        );

        let mut is_manual_minted = false;
        if witness.edit_key.is_empty() || matches!(witness.edit_value, SubAccountEditValue::Proof) {
            das_assert!(
                self.manual_mint_list_smt_root.is_some(),
                SubAccountCellErrorCode::WitnessSignMintIsRequired,
                "  witnesses[{:>2}] The account is marked as manual mint, but the manual mint list is empty.",
                witness.index
            );

            let root = self.manual_mint_list_smt_root.as_ref().unwrap();
            match smt_verify_sub_account_is_in_mint_list(root.clone(), &witness) {
                Ok(()) => {
                    debug!(
                        "  witnesses[{:>2}] The account is in the signed mint list, it can be register without payment.",
                        witness.index
                    );

                    let profit = u64::from(self.config_sub_account.new_sub_account_price()) * expiration_years;
                    self.profit_from_manual_mint += profit;
                    self.profit_total += profit;
                    self.minimal_required_das_profit +=
                        u64::from(self.config_sub_account.new_sub_account_price()) * expiration_years;

                    is_manual_minted = true;
                }
                Err(_) => {
                    if matches!(witness.edit_value, SubAccountEditValue::Proof) {
                        warn!(
                            "  witnesses[{:>2}] The proof of account is invalid, but it is marked as manual mint.",
                            witness.index
                        );

                        return Err(code_to_error!(
                            SubAccountCellErrorCode::ProofInManualSignRenewListMissing
                        ));
                    } else {
                        debug!("  witnesses[{:>2}] The account is not in the signed mint list, continue try other mint methods.", witness.index);
                    }
                }
            }
        }

        if !is_manual_minted {
            match self.flag {
                SubAccountConfigFlag::CustomScript => {
                    debug!(
                        "  witnesses[{:>2}] Record registered years and pass to custom scripts later ...",
                        witness.index
                    );

                    let mut custom_script_param = expiration_years.to_le_bytes().to_vec();
                    custom_script_param.append(&mut sub_account_reader.account().as_slice().to_vec());
                    self.custom_script_params.push(util::hex_string(&custom_script_param));

                    // This variable will be treat as the minimal profit to DAS no matter the custom script exist or not.
                    self.minimal_required_das_profit +=
                        u64::from(self.config_sub_account.new_sub_account_price()) * expiration_years;

                    das_assert!(
                        matches!(witness.edit_value, SubAccountEditValue::None),
                        SubAccountCellErrorCode::WitnessEditValueError,
                        "  witnesses[{:>2}] The edit_value should be none when the account is Custom Script Mint.",
                        witness.index
                    );
                }
                SubAccountConfigFlag::CustomRule => {
                    debug!(
                        "  witnesses[{:>2}] Execute the custome rules to check if the account is preserved and calculate its price ...",
                        witness.index
                    );

                    if self.custom_rule_flag == SubAccountCustomRuleFlag::Off {
                        warn!(
                            "  witnesses[{:>2}] The custom rules is off, the account can not be registered.",
                            witness.index
                        );
                        return Err(code_to_error!(SubAccountCellErrorCode::CustomRuleIsOff));
                    }

                    if let Some(rules) = self.custom_preserved_rules.as_ref() {
                        let matched_rule = match_rule_with_account_chars(&rules, account_chars_reader, &account)
                            .map_err(|err| {
                                warn!(
                                    "  witnesses[{:>2}] The config rules has syntax error: {}",
                                    witness.index, err
                                );
                                code_to_error!(SubAccountCellErrorCode::ConfigRulesHasSyntaxError)
                            })?;
                        if let Some(rule) = matched_rule {
                            warn!(
                                "  witnesses[{:>2}] The new SubAccount should be preserved.(matched rule: {})",
                                witness.index, rule.index
                            );
                            return Err(code_to_error!(SubAccountCellErrorCode::AccountIsPreserved));
                        }
                    }

                    if let Some(rules) = self.custom_price_rules.as_ref() {
                        let matched_rule = match_rule_with_account_chars(&rules, account_chars_reader, &account)
                            .map_err(|err| {
                                warn!(
                                    "  witnesses[{:>2}] The config rules has syntax error: {}",
                                    witness.index, err
                                );
                                code_to_error!(SubAccountCellErrorCode::ConfigRulesHasSyntaxError)
                            })?;
                        // let matched_rule = rules.last();

                        if let Some(rule) = matched_rule {
                            let profit = util::calc_yearly_capacity(rule.price, self.quote, 0) * expiration_years;

                            das_assert!(
                                profit >= u64::from(self.config_sub_account.new_sub_account_price()) * expiration_years,
                                SubAccountCellErrorCode::MinimalProfitToDASNotReached,
                                "  witnesses[{:>2}] The minimal profit to .bit should be more than {} shannon.",
                                witness.index,
                                u64::from(self.config_sub_account.new_sub_account_price()) * expiration_years
                            );

                            self.profit_total += profit;

                            debug!(
                                "  witnesses[{:>2}] account: {}, matched rule: {}, profit: {} in shannon",
                                witness.index, account, rule.index, profit
                            );
                        } else {
                            warn!(
                                "  witnesses[{:>2}] The account {} has no price, it is can not be registered.",
                                witness.index, account
                            );
                            return Err(code_to_error!(SubAccountCellErrorCode::AccountHasNoPrice));
                        }
                    } else {
                        warn!(
                            "  witnesses[{:>2}] The account {} is can not be registered, no price rule found(price_rules_hash is 0x0000...).",
                            witness.index,
                            account
                        );
                        return Err(code_to_error!(SubAccountCellErrorCode::AccountHasNoPrice));
                    }
                }
                _ => {
                    if !is_manual_minted {
                        warn!(
                            "  witnesses[{:>2}] The new SubAccount should be either manual mint or custom rule/script mint.",
                            witness.index
                        );
                        return Err(code_to_error!(SubAccountCellErrorCode::CanNotMint));
                    }
                }
            }
        }

        Ok(())
    }

    fn renew(&mut self, witness: &SubAccountWitness, prev_root: &[u8]) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();
        let new_sub_account = generate_new_sub_account_by_edit_value(witness.sub_account.clone(), &witness.edit_value)?;
        let new_sub_account_reader = new_sub_account.as_reader();

        smt_verify_sub_account_is_editable(&prev_root, &witness, new_sub_account_reader)?;

        let new_expired_at = match witness.edit_value {
            SubAccountEditValue::ExpiredAt(new_expired_at) => new_expired_at,
            _ => unreachable!(),
        };
        let expired_at = u64::from(sub_account_reader.expired_at());
        let expiration_years = (new_expired_at - expired_at) / YEAR_SEC;
        let expiration_tolerance = (new_expired_at - expired_at) % YEAR_SEC;

        debug!(
            "  witnesses[{:>2}] The account is renewed for {} years.",
            witness.index, expiration_years
        );

        das_assert!(
            expiration_years >= 1,
            SubAccountCellErrorCode::ExpirationYearsTooShort,
            "  witnesses[{:>2}] The new expired_at date should be more than 1 year after the previous expired_at date.",
            witness.index
        );

        das_assert!(
            expiration_tolerance <= DAY_SEC,
            SubAccountCellErrorCode::ExpirationToleranceReached,
            "  witnesses[{:>2}] The new expired_at date reached maximum tolerance.",
            witness.index
        );

        match (witness.edit_key.as_slice(), self.manual_renew_list_smt_root) {
            (b"manual", Some(root)) => {
                debug!(
                    "  witnesses[{:>2}] The account will be manually renewed by owner/manager.",
                    witness.index
                );

                match data_parser::sub_account_cell::get_proof_from_edit_value(&witness.edit_value_bytes) {
                    Some(proof) => {
                        if !proof.is_empty() {
                            match smt_verify_sub_account_is_in_renew_list(root.clone(), &witness) {
                                Ok(()) => {
                                    let profit =
                                        u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years;
                                    self.profit_from_manual_renew += profit;
                                    self.profit_total += profit;
                                    self.minimal_required_das_profit +=
                                        u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years;

                                    return Ok(());
                                }
                                Err(err) => {
                                    warn!(
                                        "  witnesses[{:>2}] The proof in edit_value is invalid, but it is marked as manual renew.",
                                        witness.index
                                    );

                                    return Err(err);
                                }
                            }
                        } else {
                            debug!(
                                "  witnesses[{:>2}] The account has no proof and will be treated as manually renewed by others later.",
                                witness.index
                            );
                        }
                    }
                    None => {
                        debug!(
                            "  witnesses[{:>2}] The account has no proof and will be treated as manually renewed by others later.",
                            witness.index
                        );
                    }
                }
            }
            _ => {}
        }

        match self.flag {
            SubAccountConfigFlag::CustomScript => {
                warn!(
                    "  witnesses[{:>2}] The sub-accounts with custom script are not supported for now.",
                    witness.index
                );
                return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
            }
            SubAccountConfigFlag::CustomRule => {
                if self.custom_rule_flag == SubAccountCustomRuleFlag::Off {
                    warn!(
                        "  witnesses[{:>2}] The custom rules is off, the account can not be renewed for now.",
                        witness.index
                    );
                    return Err(code_to_error!(SubAccountCellErrorCode::CustomRuleIsOff));
                }

                let sub_account_reader = witness.sub_account.as_reader();
                let (account, account_chars_reader) = gen_account_from_witness(sub_account_reader)?;

                match self.custom_price_rules.as_ref() {
                    Some(rules) => match match_rule_with_account_chars(&rules, account_chars_reader, &account) {
                        Ok(Some(rule)) => {
                            debug!(
                                "  witnesses[{:>2}] The account will be renewed with custom rules.",
                                witness.index
                            );

                            das_assert!(
                                witness.edit_key == b"custom_rule",
                                SubAccountCellErrorCode::EditKeyMismatch,
                                "  witnesses[{:>2}] The edit_key should be custom_rule, because rule[{}] found.",
                                witness.index,
                                rule.name
                            );

                            let profit = util::calc_yearly_capacity(rule.price, self.quote, 0) * expiration_years;

                            das_assert!(
                                profit
                                    >= u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years,
                                SubAccountCellErrorCode::MinimalProfitToDASNotReached,
                                "  witnesses[{:>2}] The minimal profit to .bit should be more than {} shannon.",
                                witness.index,
                                u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years
                            );

                            self.profit_total += profit;

                            debug!(
                                "  witnesses[{:>2}] account: {}, matched rule: {}, profit: {} in shannon",
                                witness.index, account, rule.index, profit
                            );

                            return Ok(());
                        }
                        Ok(None) => {
                            debug!(
                                "  witnesses[{:>2}] The account is allowed to be renewed manually because no matched rule found.",
                                witness.index
                            );
                        }
                        Err(err) => {
                            warn!(
                                "  witnesses[{:>2}] The config rules has syntax error: {}",
                                witness.index,
                                err.to_string()
                            );
                            return Err(code_to_error!(SubAccountCellErrorCode::ConfigRulesHasSyntaxError));
                        }
                    },
                    None => {
                        warn!(
                            "  witnesses[{:>2}] The account {} can not be renewed, the witness named SubAccountRules is required.",
                            witness.index, account
                        );
                        return Err(code_to_error!(SubAccountCellErrorCode::AccountHasNoPrice));
                    }
                }
                // let matched_rule = rules.last();
            }
            _ => {}
        }

        debug!(
            "  witnesses[{:>2}] The account will be manually renewed by others.",
            witness.index
        );

        das_assert!(
            witness.edit_key == b"manual",
            SubAccountCellErrorCode::EditKeyMismatch,
            "  witnesses[{:>2}] The edit_key should be manual, because it does not match any other conditions.",
            witness.index
        );

        let profit = u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years;
        self.profit_from_manual_renew_by_other += profit;
        self.profit_total += profit;
        self.minimal_required_das_profit +=
            u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years;

        Ok(())
    }

    fn edit(
        &mut self,
        witness: &SubAccountWitness,
        prev_root: &[u8],
        witness_parser: &SubAccountWitnessesParser,
    ) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();
        let new_sub_account = generate_new_sub_account_by_edit_value(witness.sub_account.clone(), &witness.edit_value)?;
        let new_sub_account_reader = new_sub_account.as_reader();

        debug!(
            "  witnesses[{:>2}] Calculated new sub-account structure is: {}",
            witness.index,
            new_sub_account_reader.as_prettier()
        );

        smt_verify_sub_account_is_editable(&prev_root, &witness, new_sub_account_reader)?;

        verifiers::sub_account_cell::verify_unlock_role(&witness)?;
        verifiers::sub_account_cell::verify_sub_account_edit_sign_not_expired(
            &witness,
            self.parent_expired_at,
            self.sub_account_last_updated_at,
        )?;
        verifiers::sub_account_cell::verify_sub_account_edit_sign(&witness, &self.sign_lib, witness_parser)?;
        verifiers::sub_account_cell::verify_expiration(
            self.config_account,
            witness.index,
            sub_account_reader,
            self.timestamp,
        )
        .map_err(|err| code_to_error!(err))?;
        verifiers::sub_account_cell::verify_status(witness.index, sub_account_reader, AccountStatus::Normal)?;

        match &witness.edit_value {
            SubAccountEditValue::Owner(new_args) | SubAccountEditValue::Manager(new_args) => {
                let current_args = sub_account_reader.lock().args().raw_data();
                let (current_owner_type, current_owner_args, current_manager_type, current_manager_args) =
                    data_parser::das_lock_args::get_owner_and_manager(current_args)?;
                let (new_owner_type, new_owner_args, new_manager_type, new_manager_args) =
                    data_parser::das_lock_args::get_owner_and_manager(new_args)?;

                if let SubAccountEditValue::Owner(_) = &witness.edit_value {
                    debug!(
                        "  witnesses[{:>2}] Verify if owner has been changed correctly.",
                        witness.index
                    );

                    das_assert!(
                        current_owner_type != new_owner_type || current_owner_args != new_owner_args,
                        SubAccountCellErrorCode::SubAccountEditLockError,
                        "  witnesses[{:>2}] The owner fields in args should be consistent.",
                        witness.index
                    );

                    // Skip verifying manger, because owner has been changed.
                } else {
                    debug!(
                        "  witnesses[{:>2}] Verify if manager has been changed correctly.",
                        witness.index
                    );

                    das_assert!(
                        current_owner_type == new_owner_type && current_owner_args == new_owner_args,
                        SubAccountCellErrorCode::SubAccountEditLockError,
                        "  witnesses[{:>2}] The owner fields in args should be consistent.",
                        witness.index
                    );

                    das_assert!(
                        current_manager_type != new_manager_type || current_manager_args != new_manager_args,
                        SubAccountCellErrorCode::SubAccountEditLockError,
                        "  witnesses[{:>2}] The manager fields in args should be changed.",
                        witness.index
                    );
                }
            }
            SubAccountEditValue::Records(records) => {
                verifiers::account_cell::verify_records_keys(self.parser, records.as_reader())?;
            }
            // manual::verify_edit_value_not_empty
            SubAccountEditValue::None | _ => {
                warn!(
                    "  witnesses[{:>2}] The witness.edit_value should not be empty.",
                    witness.index
                );
                return Err(code_to_error!(SubAccountCellErrorCode::SubAccountFieldNotEditable));
            }
        }

        Ok(())
    }

    fn recycle(&mut self, witness: &SubAccountWitness, prev_root: &[u8]) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();

        match verifiers::sub_account_cell::verify_expiration(
            self.config_account,
            witness.index,
            sub_account_reader,
            self.timestamp,
        ) {
            Ok(_) => {
                warn!(
                    "  witnesses[{:>2}] The sub-account is not expired, can not be recycled.",
                    witness.index
                );
                return Err(code_to_error!(SubAccountCellErrorCode::AccountStillCanNotBeRecycled));
            }
            Err(SubAccountCellErrorCode::AccountHasInGracePeriod) => {
                warn!(
                    "  witnesses[{:>2}] The sub-account is in expiration grace period , can be recycled.",
                    witness.index
                );
                return Err(code_to_error!(SubAccountCellErrorCode::AccountStillCanNotBeRecycled));
            }
            Err(SubAccountCellErrorCode::AccountHasExpired) => {
                debug!(
                    "  witnesses[{:>2}] The sub-account is expired, can be recycled.",
                    witness.index
                );
            }
            _ => {
                // This branch should be unreachable.
                return Err(code_to_error!(ErrorCode::HardCodedError));
            }
        }

        smt_verify_sub_account_is_removed(&prev_root, &witness)?;

        Ok(())
    }
}

fn gen_account_from_witness(
    sub_account_reader: SubAccountReader,
) -> Result<(String, AccountCharsReader), Box<dyn ScriptError>> {
    let account_chars_reader = sub_account_reader.account();
    let mut account_bytes = account_chars_reader.as_readable();
    account_bytes.extend(sub_account_reader.suffix().raw_data());
    let account =
        String::from_utf8(account_bytes).map_err(|_| code_to_error!(SubAccountCellErrorCode::BytesToStringFailed))?;

    Ok((account, account_chars_reader))
}

fn gen_smt_key_by_account_id(account_id: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let key_pre = [account_id, &[0u8; 12]].concat();
    key.copy_from_slice(&key_pre);
    key
}

fn smt_verify_sub_account_is_in_mint_list(
    root: [u8; 32],
    witness: &SubAccountWitness,
) -> Result<(), Box<dyn ScriptError>> {
    // TODO Unify the error codes here with the renew action
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

fn smt_verify_sub_account_is_in_renew_list(
    root: [u8; 32],
    witness: &SubAccountWitness,
) -> Result<(), Box<dyn ScriptError>> {
    let proof = match data_parser::sub_account_cell::get_proof_from_edit_value(&witness.edit_value_bytes) {
        Some(proof) => proof,
        None => return Err(code_to_error!(SubAccountCellErrorCode::ManualRenewProofIsRequired)),
    };
    let key = gen_smt_key_by_account_id(witness.sub_account.id().as_slice());
    let value = util::blake2b_256(witness.sub_account.lock().args().as_reader().raw_data());

    debug!(
        "  witnesses[{:>2}] Verify if {} is exist in the SubAccountMintSignWitness.account_list_smt_root.(key: 0x{})",
        witness.index,
        witness.sub_account.account().as_prettier(),
        util::hex_string(&key)
    );

    verifiers::common::verify_smt_proof(key, value, root, proof)
        .map_err(|_| code_to_error!(SubAccountCellErrorCode::ManualRenewProofIsInvalid))?;

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

fn smt_verify_sub_account_is_removed(
    prev_root: &[u8],
    witness: &SubAccountWitness,
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
    let current_val = [0u8; 32];
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
            let mut sub_account_builder = sub_account.as_builder();
            sub_account_builder = sub_account_builder.expired_at(Uint64::from(val.to_owned()));

            sub_account_builder
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
        _ => return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid)),
    };

    // Every time a sub-account is edited, its nonce must  increase by 1 .
    sub_account_builder = sub_account_builder.nonce(Uint64::from(current_nonce + 1));

    Ok(sub_account_builder.build())
}
