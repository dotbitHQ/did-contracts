use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use das_core::constants::*;
use das_core::error::{ErrorCode, ScriptError, SubAccountCellErrorCode};
use das_core::util::{self, blake2b_256};
use das_core::witness_parser::sub_account::{SubAccountEditValue, SubAccountWitness, SubAccountWitnessesParser};
use das_core::{code_to_error, das_assert, data_parser, debug, verifiers, warn};
use das_types::constants::{das_lock, *};
use das_types::mixer::SubAccountReaderMixer;
use das_types::packed::*;
use das_types::prelude::{Builder, Entity};
#[cfg(debug_assertions)]
use das_types::prettier::Prettier;
use simple_ast::executor::match_rule_with_account_chars;
use simple_ast::types as ast_types;

use super::approval;

pub struct SubAction<'a> {
    //sign_lib: SignLib,
    timestamp: u64,
    quote: u64,
    flag: SubAccountConfigFlag,
    custom_rule_flag: SubAccountCustomRuleFlag,
    sub_account_last_updated_at: u64,

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

    // custom rule fields
    custom_preserved_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
    custom_price_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
    is_custom_price_rules_set: bool,
}

impl<'a> SubAction<'a> {
    pub fn new(
        //sign_lib: SignLib,
        timestamp: u64,
        quote: u64,
        flag: SubAccountConfigFlag,
        custom_rule_flag: SubAccountCustomRuleFlag,
        sub_account_last_updated_at: u64,
        config_account: ConfigCellAccountReader<'a>,
        config_sub_account: ConfigCellSubAccountReader<'a>,
        parent_account: &'a [u8],
        parent_expired_at: u64,
        manual_mint_list_smt_root: &'a Option<[u8; 32]>,
        manual_renew_list_smt_root: &'a Option<[u8; 32]>,
        custom_preserved_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
        custom_price_rules: &'a Option<Vec<ast_types::SubAccountRule>>,
        is_custom_price_rules_set: bool,
    ) -> Self {
        Self {
            //sign_lib,
            timestamp,
            quote,
            flag,
            custom_rule_flag,
            sub_account_last_updated_at,
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
            custom_preserved_rules,
            custom_price_rules,
            is_custom_price_rules_set,
        }
    }

    pub fn dispatch(
        &mut self,
        witness: &SubAccountWitness,
        prev_root: &[u8],
        _witness_parser: &SubAccountWitnessesParser,
    ) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();

        verifiers::sub_account_cell::verify_suffix_with_parent_account(
            witness.index,
            &sub_account_reader,
            self.parent_account,
        )?;

        debug!(
            "  witnesses[{:>2}] Start verify {} action ...",
            witness.index,
            witness.action.to_string()
        );

        match witness.action {
            SubAccountAction::Create => self.create(witness, prev_root)?,
            SubAccountAction::Renew => self.renew(witness, prev_root)?,
            SubAccountAction::Edit => self.edit(witness, prev_root)?,
            SubAccountAction::Recycle => self.recycle(witness, prev_root)?,
            SubAccountAction::CreateApproval
            | SubAccountAction::DelayApproval
            | SubAccountAction::RevokeApproval
            | SubAccountAction::FulfillApproval => self.approve(witness, prev_root)?,
        }

        Ok(())
    }

    fn create(&mut self, witness: &SubAccountWitness, prev_root: &[u8]) -> Result<(), Box<dyn ScriptError>> {
        smt_verify_sub_account_is_creatable(&prev_root, &witness)?;

        debug!(
            "  witnesses[{:>2}] Verify if the account is registrable.",
            witness.index
        );

        das_assert!(
            witness.old_sub_account_version == 2 && witness.new_sub_account_version == 2,
            SubAccountCellErrorCode::WitnessVersionMismatched,
            "  witnesses[{:>2}] The old_sub_account_version and new_sub_account_version should be 2.",
            witness.index
        );

        let sub_account = match witness.sub_account.try_into_latest() {
            Ok(sub_account) => sub_account,
            Err(_) => {
                debug!(
                    "  witnesses[{:>2}] The new SubAccount should be the latest version.",
                    witness.index
                );
                return Err(code_to_error!(SubAccountCellErrorCode::WitnessVersionMismatched));
            }
        };
        let sub_account_reader = sub_account.as_reader();
        let sub_account_reader_mixer = witness.sub_account.as_reader();

        let (account, account_chars_reader) = gen_account_from_witness(&sub_account_reader_mixer)?;

        verifiers::account_cell::verify_account_chars(account_chars_reader)?;
        verifiers::account_cell::verify_account_chars_min_length(account_chars_reader)?;
        verifiers::account_cell::verify_account_chars_max_length(account_chars_reader)?;

        verifiers::sub_account_cell::verify_initial_properties(witness.index, sub_account_reader, self.timestamp)?;

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

                    let profit = calc_basic_profit(
                        self.config_sub_account.new_sub_account_price(),
                        self.quote,
                        expiration_years,
                    );
                    self.profit_from_manual_mint += profit;
                    self.profit_total += profit;
                    self.minimal_required_das_profit += profit;

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
                            das_assert!(
                                rule.price >= u64::from(self.config_sub_account.new_sub_account_price()),
                                SubAccountCellErrorCode::MinimalProfitToDASNotReached,
                                "  witnesses[{:>2}] The minimal profit to .bit should be more than {} shannon.",
                                witness.index,
                                u64::from(self.config_sub_account.new_sub_account_price()) * expiration_years
                            );

                            let profit = util::calc_yearly_capacity(rule.price, self.quote, 0) * expiration_years;
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
        let new_sub_account = generate_new_sub_account_by_edit_value(&witness)?;
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

        let mut manually_renew_by_others = true;
        let sub_account_reader = witness.sub_account.as_reader();
        let (account, account_chars_reader) = gen_account_from_witness(&sub_account_reader)?;

        // WARNING! The `manual` renew has been the fallback option for all flags now:
        // If the flag is `custom_rule`, any user can manually renew their sub-account in two situations:
        //   1. The sub-account do not match any exist rules.
        //   2. The custom_rule is turned off.
        // In both situations, the `flag` will be ignored. Even if the `edit_key` is set to `custom_rule`, we will still treat it as a manual renewal.
        // If the flag is set to 'manual', any user can manually renew their sub-account at any time.
        match (witness.edit_key.as_slice(), self.flag) {
            (b"custom_rule", SubAccountConfigFlag::CustomRule) => {
                if self.custom_rule_flag == SubAccountCustomRuleFlag::On {
                    match self.custom_price_rules.as_ref() {
                        Some(rules) => match match_rule_with_account_chars(&rules, account_chars_reader, &account) {
                            Ok(Some(rule)) => {
                                debug!(
                                    "  witnesses[{:>2}] The account will be renewed with custom rules.",
                                    witness.index
                                );

                                das_assert!(
                                    rule.price >= u64::from(self.config_sub_account.renew_sub_account_price()),
                                    SubAccountCellErrorCode::MinimalProfitToDASNotReached,
                                    "  witnesses[{:>2}] The minimal profit to .bit should be more than {} shannon.",
                                    witness.index,
                                    u64::from(self.config_sub_account.renew_sub_account_price()) * expiration_years
                                );

                                let profit = util::calc_yearly_capacity(rule.price, self.quote, 0) * expiration_years;
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
                            if self.is_custom_price_rules_set {
                                warn!(
                                    "  witnesses[{:>2}] The account {} can not be renewed, the witness named SubAccountRules is required.",
                                    witness.index, account
                                );
                                return Err(code_to_error!(SubAccountCellErrorCode::AccountHasNoPrice));
                            } else {
                                debug!(
                                    "  witnesses[{:>2}] The account is allowed to be renewed manually because no rule is set.",
                                    witness.index
                                );
                            }
                        }
                    }
                } else {
                    debug!(
                        "  witnesses[{:>2}] The custom rules is off, the account can be renewed by anyone now.",
                        witness.index
                    );
                }
            }
            // No matter what the flag is, the owner and manager can always manually renew so we use _ match here.
            (b"manual", _) => {
                if let Some(root) = self.manual_renew_list_smt_root {
                    // The signature is still reqired to verify the spending of owner/manager's BalanceCell.
                    match data_parser::sub_account_cell::get_proof_from_edit_value(&witness.edit_value_bytes) {
                        Some(proof) => {
                            if !proof.is_empty() {
                                debug!(
                                    "  witnesses[{:>2}] The account will be manually renewed by owner/manager.",
                                    witness.index
                                );

                                match smt_verify_sub_account_is_in_renew_list(root.clone(), &witness) {
                                    Ok(()) => {
                                        manually_renew_by_others = false;
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
                                        "  witnesses[{:>2}] The account has no proof and will be treated as manually renewed by others.",
                                        witness.index
                                    );
                            }
                        }
                        None => {
                            debug!(
                                    "  witnesses[{:>2}] The account has no proof and will be treated as manually renewed by others.",
                                    witness.index
                                );
                        }
                    }
                } else {
                    debug!(
                        "  witnesses[{:>2}] The account will be treated as manually renewed by others.",
                        witness.index
                    );
                }
            }
            _ => {
                let sub_account_reader = witness.sub_account.as_reader();
                let (account, _) = gen_account_from_witness(&sub_account_reader)?;

                warn!(
                    "  witnesses[{:>2}] The account {} renew failed, unknown combination of {} and edit_key .",
                    witness.index,
                    account,
                    self.flag.to_string()
                );
                return Err(code_to_error!(SubAccountCellErrorCode::AccountHasNoPrice));
            }
        }

        let profit = calc_basic_profit(
            self.config_sub_account.renew_sub_account_price(),
            self.quote,
            expiration_years,
        );
        if !manually_renew_by_others {
            self.profit_from_manual_renew += profit;
        } else {
            self.profit_from_manual_renew_by_other += profit;
        }
        self.profit_total += profit;
        self.minimal_required_das_profit += profit;

        debug!(
            "  witnesses[{:>2}] account: {}, manually renew, profit: {} in shannon",
            witness.index, account, profit
        );

        Ok(())
    }

    fn edit(&mut self, witness: &SubAccountWitness, prev_root: &[u8]) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();
        let new_sub_account = generate_new_sub_account_by_edit_value(&witness)?;
        let new_sub_account_reader = new_sub_account.as_reader();

        debug!(
            "  witnesses[{:>2}] Calculated new sub-account structure is: {}",
            witness.index,
            Prettier::as_prettier(&new_sub_account_reader)
        );

        smt_verify_sub_account_is_editable(&prev_root, &witness, new_sub_account_reader)?;

        verifiers::sub_account_cell::verify_unlock_role(&witness)?;
        verifiers::sub_account_cell::verify_sub_account_edit_sign_not_expired(
            &witness,
            self.parent_expired_at,
            self.sub_account_last_updated_at,
        )?;
        // verifiers::sub_account_cell::verify_sub_account_edit_sign(&witness, &self.sign_lib, witness_parser)?;
        verifiers::sub_account_cell::verify_expiration(
            self.config_account,
            witness.index,
            &sub_account_reader,
            self.timestamp,
        )
        .map_err(|err| code_to_error!(err))?;

        match &witness.edit_value {
            SubAccountEditValue::Owner(new_args) | SubAccountEditValue::Manager(new_args) => {
                verifiers::sub_account_cell::verify_status(witness.index, &sub_account_reader, AccountStatus::Normal)?;

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
                verifiers::sub_account_cell::verify_status_v2(
                    witness.index,
                    &sub_account_reader,
                    &[AccountStatus::Normal, AccountStatus::ApprovedTransfer],
                )?;

                verifiers::account_cell::verify_records_keys(records.as_reader())?;
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

        // WARNING! The sub-account only has 2 status for now, if more status added, the recycling logic should be also updated.
        verifiers::sub_account_cell::verify_status_v2(
            witness.index,
            &sub_account_reader,
            &[AccountStatus::Normal, AccountStatus::ApprovedTransfer],
        )?;

        match verifiers::sub_account_cell::verify_expiration(
            self.config_account,
            witness.index,
            &sub_account_reader,
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

    fn approve(&mut self, witness: &SubAccountWitness, prev_root: &[u8]) -> Result<(), Box<dyn ScriptError>> {
        let sub_account_reader = witness.sub_account.as_reader();
        let new_sub_account = generate_new_sub_account_by_edit_value(&witness)?;
        let new_sub_account_reader = new_sub_account.as_reader();

        debug!(
            "  witnesses[{:>2}] Calculated new sub-account structure is: {}",
            witness.index,
            Prettier::as_prettier(&new_sub_account_reader)
        );

        smt_verify_sub_account_is_editable(&prev_root, &witness, new_sub_account_reader)?;

        let approval_reader = match witness.action {
            SubAccountAction::CreateApproval => new_sub_account_reader.approval(),
            _ => {
                let sub_account_reader = sub_account_reader
                    .try_into_latest()
                    .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessVersionMismatched))?;
                sub_account_reader.approval()
            }
        };
        let approval_action = approval_reader.action().raw_data();
        let approval_params = approval_reader.params().raw_data();

        debug!("  witnesses[{:>2}] Verify if the signature is valid.", witness.index);

        match witness.action {
            SubAccountAction::CreateApproval | SubAccountAction::DelayApproval => {
                verifiers::sub_account_cell::verify_unlock_role(&witness)?;
                verifiers::sub_account_cell::verify_sub_account_edit_sign_not_expired(
                    &witness,
                    self.parent_expired_at,
                    self.sub_account_last_updated_at,
                )?;
                //verifiers::sub_account_cell::verify_sub_account_edit_sign(&witness, &self.sign_lib, witness_parser)?;
            }
            SubAccountAction::RevokeApproval => {
                verifiers::sub_account_cell::verify_sub_account_edit_sign_not_expired(
                    &witness,
                    self.parent_expired_at,
                    self.sub_account_last_updated_at,
                )?;
                // verifiers::sub_account_cell::verify_sub_account_approval_sign(
                //     &witness,
                //     &self.sign_lib,
                //     witness_parser,
                // )?;
            }
            SubAccountAction::FulfillApproval => match approval_action {
                b"transfer" => {
                    let params = AccountApprovalTransferReader::from_compatible_slice(approval_params)
                        .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessParsingError))?;
                    let sealed_util = u64::from(params.sealed_until());

                    if self.timestamp <= sealed_util {
                        debug!(
                            "  witnesses[{:>2}] The approval is sealed, verify the signature with the owner lock.",
                            witness.index
                        );

                        verifiers::sub_account_cell::verify_sub_account_edit_sign_not_expired(
                            &witness,
                            self.parent_expired_at,
                            self.sub_account_last_updated_at,
                        )?;
                        // verifiers::sub_account_cell::verify_sub_account_approval_sign(
                        //     &witness,
                        //     &self.sign_lib,
                        //     witness_parser,
                        // )?;
                    } else {
                        debug!(
                            "  witnesses[{:>2}] The approval is released, no need to verify the signature.",
                            witness.index
                        );
                    }
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
            },
            _ => {
                warn!(
                    "  witnesses[{:>2}] The action is not an approval actions.",
                    witness.index
                );
                return Err(code_to_error!(SubAccountCellErrorCode::SignError));
            }
        }

        debug!("  witnesses[{:>2}] Get the approval action.", witness.index);

        match witness.action {
            SubAccountAction::CreateApproval => match approval_action {
                b"transfer" => {
                    approval::transfer_approval_create(
                        witness.index,
                        self.timestamp,
                        sub_account_reader,
                        new_sub_account_reader,
                    )?;
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
            },
            SubAccountAction::DelayApproval => match approval_action {
                b"transfer" => {
                    let sub_account_reader = sub_account_reader
                        .try_into_latest()
                        .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessVersionMismatched))?;
                    approval::transfer_approval_delay(
                        witness.index,
                        sub_account_reader.approval(),
                        new_sub_account_reader.approval(),
                    )?;
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
            },
            SubAccountAction::RevokeApproval => match approval_action {
                b"transfer" => {
                    let sub_account_reader = sub_account_reader
                        .try_into_latest()
                        .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessVersionMismatched))?;
                    approval::transfer_approval_revoke(
                        witness.index,
                        self.timestamp,
                        sub_account_reader.approval(),
                        new_sub_account_reader,
                    )?;
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
            },
            SubAccountAction::FulfillApproval => match approval_action {
                b"transfer" => {
                    debug!("  witnesses[{:>2}] The SMT verification has ensured the sub-account transfered properly, so no more verifications here.", witness.index);
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
            },
            _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
        }

        Ok(())
    }
}

fn gen_account_from_witness<'a>(
    sub_account_reader: &'a Box<dyn SubAccountReaderMixer + 'a>,
) -> Result<(String, AccountCharsReader<'a>), Box<dyn ScriptError>> {
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
    let sub_account_reader = witness.sub_account.as_reader();
    let key = gen_smt_key_by_account_id(sub_account_reader.id().as_slice());
    let value = util::blake2b_256(sub_account_reader.lock().args().raw_data());

    debug!(
        "  witnesses[{:>2}] Verify if {} is exist in the SubAccountMintSignWitness.account_list_smt_root.(key: 0x{})",
        witness.index,
        sub_account_reader.account().as_prettier(),
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
    let sub_account_reader = witness.sub_account.as_reader();
    let key = gen_smt_key_by_account_id(sub_account_reader.id().as_slice());
    let value = util::blake2b_256(sub_account_reader.lock().args().raw_data());

    debug!(
        "  witnesses[{:>2}] Verify if {} is exist in the SubAccountMintSignWitness.account_list_smt_root.(key: 0x{})",
        witness.index,
        sub_account_reader.account().as_prettier(),
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
    let sub_account_reader = witness.sub_account.as_reader();
    let key = gen_smt_key_by_account_id(sub_account_reader.id().as_slice());
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
    let current_val = blake2b_256(sub_account_reader.as_slice()).to_vec().try_into().unwrap();
    // debug!("current_val_prettier = {}", sub_account_reader.as_prettier());
    verifiers::common::verify_smt_proof(key, current_val, current_root.try_into().unwrap(), proof)?;

    Ok(())
}

fn smt_verify_sub_account_is_editable<'a>(
    prev_root: &[u8],
    witness: &SubAccountWitness,
    new_sub_account: SubAccountReader,
) -> Result<(), Box<dyn ScriptError>> {
    let sub_account_reader = witness.sub_account.as_reader();
    let key = gen_smt_key_by_account_id(sub_account_reader.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!(
        "  witnesses[{:>2}] Verify if the current state of the sub-account was in the SMT before.(key: 0x{})",
        witness.index,
        util::hex_string(&key)
    );
    let prev_val: [u8; 32] = blake2b_256(sub_account_reader.as_slice()).to_vec().try_into().unwrap();
    // debug!("prev_val = 0x{}", util::hex_string(&prev_val));
    // debug!("prev_val_raw = 0x{}", util::hex_string(witness.sub_account.as_slice()));
    // debug!("prev_val_prettier = {}", witness.sub_account.as_prettier());
    verifiers::common::verify_smt_proof(key, prev_val, prev_root.try_into().unwrap(), proof)?;

    debug!(
        "  witnesses[{:>2}] Verify if the new state of the sub-account is in the SMT now.",
        witness.index
    );
    let current_root = witness.new_root.as_slice();
    let current_val: [u8; 32] = blake2b_256(Reader::as_slice(&new_sub_account))
        .to_vec()
        .try_into()
        .unwrap();
    // debug!("current_val = 0x{}", util::hex_string(&current_val));
    // debug!("current_val_raw = 0x{}", util::hex_string(new_sub_account.as_slice()));
    // debug!("current_val_prettier = {}", Prettier::as_prettier(&new_sub_account));
    verifiers::common::verify_smt_proof(key, current_val, current_root.try_into().unwrap(), proof)?;

    Ok(())
}

fn smt_verify_sub_account_is_removed(
    prev_root: &[u8],
    witness: &SubAccountWitness,
) -> Result<(), Box<dyn ScriptError>> {
    let sub_account_reader = witness.sub_account.as_reader();
    let key = gen_smt_key_by_account_id(sub_account_reader.id().as_slice());
    let proof = witness.proof.as_slice();

    debug!(
        "  witnesses[{:>2}] Verify if the current state of the sub-account was in the SMT before.(key: 0x{})",
        witness.index,
        util::hex_string(&key)
    );
    let prev_val: [u8; 32] = blake2b_256(sub_account_reader.as_slice()).to_vec().try_into().unwrap();
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

fn generate_new_sub_account_by_edit_value(witness: &SubAccountWitness) -> Result<SubAccount, Box<dyn ScriptError>> {
    das_assert!(
        witness.new_sub_account_version == 2,
        SubAccountCellErrorCode::WitnessUpgradeNeeded,
        "  witnesses[{:>2}] SubAccount.new_sub_account_version is invalid.(expected: {}, actual: {})",
        witness.index,
        2,
        witness.new_sub_account_version
    );

    // Upgrade the earlier version to the latest version, because the new SubAccount should always be kept up to date.
    let sub_account = witness.sub_account.clone();
    let sub_account = if sub_account.version() == 1 {
        let sub_account = sub_account
            .try_into_v1()
            .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessVersionMismatched))?;

        SubAccount::new_builder()
            .lock(sub_account.lock().clone())
            .id(sub_account.id().clone())
            .account(sub_account.account().clone())
            .suffix(sub_account.suffix().clone())
            .registered_at(sub_account.registered_at().clone())
            .expired_at(sub_account.expired_at().clone())
            .status(sub_account.status().clone())
            .records(sub_account.records().clone())
            .nonce(sub_account.nonce().clone())
            .enable_sub_account(sub_account.enable_sub_account().clone())
            .renew_sub_account_price(sub_account.renew_sub_account_price().clone())
            .build()
    } else {
        sub_account
            .try_into_latest()
            .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessVersionMismatched))?
    };

    let edit_value = &witness.edit_value;

    let current_nonce = u64::from(sub_account.nonce());
    let current_approval = sub_account.approval().clone();
    let current_approval_reader = current_approval.as_reader();
    let mut sub_account_builder = sub_account.as_builder();
    sub_account_builder = match witness.action {
        SubAccountAction::Edit => {
            match edit_value {
                SubAccountEditValue::Owner(val) | SubAccountEditValue::Manager(val) => {
                    let mut lock_builder = das_lock().clone().as_builder();
                    // Verify if the edit_value is a valid format.
                    data_parser::das_lock_args::get_owner_and_manager(val)?;
                    lock_builder = lock_builder.args(Bytes::from(val.to_owned()));

                    sub_account_builder = sub_account_builder.lock(lock_builder.build());

                    if let SubAccountEditValue::Owner(_) = edit_value {
                        sub_account_builder = sub_account_builder.records(Records::default())
                    }

                    sub_account_builder
                }
                SubAccountEditValue::Records(val) => sub_account_builder.records(val.to_owned()),
                _ => return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid)),
            }
        }
        SubAccountAction::Renew => match edit_value {
            SubAccountEditValue::ExpiredAt(val) => sub_account_builder.expired_at(Uint64::from(val.to_owned())),
            _ => return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid)),
        },
        SubAccountAction::CreateApproval | SubAccountAction::DelayApproval => {
            match edit_value {
                SubAccountEditValue::Approval(val) => {
                    // The status should be updated to AccountStatus::ApprovedTransfer when the edit_value is approval.
                    sub_account_builder =
                        sub_account_builder.status(Uint8::from(AccountStatus::ApprovedTransfer as u8));
                    sub_account_builder.approval(val.to_owned())
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid)),
            }
        }
        SubAccountAction::RevokeApproval => {
            match edit_value {
                SubAccountEditValue::None => {}
                _ => {
                    return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid));
                }
            }

            // The status should be updated to AccountStatus::Normal when the edit_value is None.
            sub_account_builder = sub_account_builder.status(Uint8::from(AccountStatus::Normal as u8));
            sub_account_builder.approval(AccountApproval::default())
        }
        SubAccountAction::FulfillApproval => {
            match edit_value {
                SubAccountEditValue::None => {}
                _ => {
                    return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid));
                }
            }

            let approval_action = current_approval_reader.action().raw_data();
            let approval_params = current_approval_reader.params().raw_data();

            match approval_action {
                b"transfer" => {
                    let approval_params_reader = AccountApprovalTransferReader::from_compatible_slice(approval_params)
                        .map_err(|_| code_to_error!(SubAccountCellErrorCode::WitnessParsingError))?;
                    sub_account_builder = sub_account_builder.lock(approval_params_reader.to_lock().to_entity());
                    sub_account_builder = sub_account_builder.records(Records::default());
                    // The status should be updated to AccountStatus::Normal when the edit_value is None.
                    sub_account_builder = sub_account_builder.status(Uint8::from(AccountStatus::Normal as u8));
                    sub_account_builder.approval(AccountApproval::default())
                }
                _ => return Err(code_to_error!(SubAccountCellErrorCode::ApprovalActionUndefined)),
            }
        }
        _ => return Err(code_to_error!(SubAccountCellErrorCode::WitnessEditKeyInvalid)),
    };

    // Every time a sub-account is edited, its nonce must  increase by 1 .
    sub_account_builder = sub_account_builder.nonce(Uint64::from(current_nonce + 1));

    Ok(sub_account_builder.build())
}

fn calc_basic_profit(yearly_price: Uint64Reader, quote: u64, expiration_years: u64) -> u64 {
    let usd_price = u64::from(yearly_price);
    let ckb_price = util::calc_yearly_capacity(usd_price, quote, 0);
    let profit = ckb_price * expiration_years;

    profit
}
