use std::convert::TryFrom;
use std::str::FromStr;

use das_types_std::constants::*;
use das_types_std::mixer::SubAccountMixer;
use das_types_std::packed::*;
use das_types_std::prelude::*;
use das_types_std::util as das_util;
use serde_json::Value;

use super::super::smt::SMTWithHistory;
use super::super::util;
use super::util as encoder_util;

pub fn to_v1(path: &str, value: &Value) -> SubAccountV1 {
    let (
        suffix,
        lock,
        account_id,
        account_chars,
        registered_at,
        expired_at,
        status,
        records,
        nonce,
        enable_sub_account,
        renew_sub_account_price,
    ) = encode_v1_fields(path, value);

    SubAccountV1::new_builder()
        .lock(lock)
        .id(account_id)
        .account(account_chars)
        .suffix(Bytes::from(suffix.as_bytes()))
        .registered_at(registered_at)
        .expired_at(expired_at)
        .status(status)
        .records(records)
        .nonce(nonce)
        .enable_sub_account(enable_sub_account)
        .renew_sub_account_price(renew_sub_account_price)
        .build()
}

pub fn to_latest(path: &str, value: &Value) -> SubAccount {
    let (
        suffix,
        lock,
        account_id,
        account_chars,
        registered_at,
        expired_at,
        status,
        records,
        nonce,
        enable_sub_account,
        renew_sub_account_price,
    ) = encode_v1_fields(path, value);
    let approval = encode_v2_fields(&format!("{}.approval", path), &value["approval"]);

    let entity = SubAccount::new_builder()
        .lock(lock)
        .id(account_id)
        .account(account_chars)
        .suffix(Bytes::from(suffix.as_bytes()))
        .registered_at(registered_at)
        .expired_at(expired_at)
        .status(status)
        .records(records)
        .nonce(nonce)
        .enable_sub_account(enable_sub_account)
        .renew_sub_account_price(renew_sub_account_price)
        .approval(approval)
        .build();
    // println!("entity = {}", entity.as_prettier());

    entity
}

fn encode_v1_fields(
    path: &str,
    value: &Value,
) -> (
    String,
    Script,
    AccountId,
    AccountChars,
    Uint64,
    Uint64,
    Uint8,
    Records,
    Uint64,
    Uint8,
    Uint64,
) {
    let lock = util::parse_json_script_to_mol(
        "",
        &util::parse_json_script_das_lock(&format!("{}.lock", path), &value["lock"]),
    );
    let suffix = util::parse_json_str(&format!("{}.suffix", path), &value["suffix"]).to_string();
    let (account, account_chars) =
        util::parse_json_to_account_chars(&format!("{}.account", path), &value["account"], Some(&suffix));
    let account_id = if !value["id"].is_null() {
        util::parse_json_str_to_account_id_mol(&format!("{}.id", path), &value["id"])
    } else {
        AccountId::try_from(util::account_to_id(&account)).expect("Calculate account ID from account failed")
    };
    let registered_at = Uint64::from(util::parse_json_u64(
        &format!("{}.registered_at", path),
        &value["registered_at"],
        None,
    ));
    let expired_at = Uint64::from(util::parse_json_u64(
        &format!("{}.expired_at", path),
        &value["expired_at"],
        None,
    ));
    let status = Uint8::from(util::parse_json_u8(
        &format!("{}.status", path),
        &value["status"],
        Some(0),
    ));
    let records = util::parse_json_to_records_mol(&format!("{}.records", path), &value["records"]);
    let nonce = Uint64::from(util::parse_json_u64(
        &format!("{}.nonce", path),
        &value["nonce"],
        Some(0),
    ));
    let enable_sub_account = Uint8::from(util::parse_json_u8(
        &format!("{}.enable_sub_account", path),
        &value["enable_sub_account"],
        Some(0),
    ));
    let renew_sub_account_price = Uint64::from(util::parse_json_u64(
        &format!("{}.renew_sub_account_price", path),
        &value["renew_sub_account_price"],
        Some(0),
    ));

    (
        suffix,
        lock,
        account_id,
        account_chars,
        registered_at,
        expired_at,
        status,
        records,
        nonce,
        enable_sub_account,
        renew_sub_account_price,
    )
}

fn encode_v2_fields(path: &str, value: &Value) -> AccountApproval {
    if value.is_null() {
        return AccountApproval::default();
    }

    let approval_action = util::parse_json_str(&format!("{}.action", path), &value["action"]);
    let approval_params = match approval_action {
        // "transfer" => {
        // This is use for providing invalid action
        _ => {
            let platform_lock = util::parse_json_script_to_mol(
                &format!("{}.params.platform_lock", path),
                &value["params"]["platform_lock"],
            );
            let protected_until = util::parse_json_u64(
                &format!("{}.params.protected_until", path),
                &value["params"]["protected_until"],
                None,
            );
            let sealed_until = util::parse_json_u64(
                &format!("{}.params.sealed_until", path),
                &value["params"]["sealed_until"],
                None,
            );
            let delay_count_remain = util::parse_json_u8(
                &format!("{}.params.delay_count_remain", path),
                &value["params"]["delay_count_remain"],
                None,
            );
            let to_lock = util::parse_json_script_to_mol(
                &format!("{}.params.to_lock", path),
                &value["params"]["to_lock"]
            );
            let account_approval_transfer = AccountApprovalTransfer::new_builder()
                .platform_lock(platform_lock)
                .protected_until(Uint64::from(protected_until))
                .sealed_until(Uint64::from(sealed_until))
                .delay_count_remain(Uint8::from(delay_count_remain))
                .to_lock(to_lock)
                .build();
            Bytes::from(account_approval_transfer.as_slice().to_vec())
        }
        // _ => unimplemented!("Not support action: {}", approval_action),
    };
    let approval = AccountApproval::new_builder()
        .action(Bytes::from(approval_action.as_bytes()))
        .params(approval_params)
        .build();

    approval
}

pub fn to_raw_witness_v2(smt_with_history: &mut SMTWithHistory, path: &str, value: &Value) -> Vec<u8> {
    if value["sub_account"].is_null() {
        panic!("witness.sub_account is missing");
    }

    let (action, mut witness_bytes) = encode_raw_witness_common_fields(path, value, Some(2));

    let key = get_smt_key_from_json(&format!("{}.sub_account", path), &value["sub_account"]);
    let entity = to_v1(&format!("{}.sub_account", path), &value["sub_account"]);
    let entity_bytes = Entity::as_slice(&entity).to_vec();
    let (new_root, compiled_proof) =
        get_smt_new_root_and_proof(&action, smt_with_history, path, key, value, Box::new(entity));

    encode_smt_fields(&mut witness_bytes, new_root, compiled_proof);

    witness_bytes.extend(encoder_util::length_of(&entity_bytes));
    witness_bytes.extend(entity_bytes);

    encode_edit_fields(&action, path, &mut witness_bytes, &value);

    das_util::wrap_raw_witness_v2(DataType::SubAccount, witness_bytes)
}

/// v3
pub fn to_raw_witness_latest(smt_with_history: &mut SMTWithHistory, path: &str, value: &Value) -> Vec<u8> {
    if value["sub_account"].is_null() {
        panic!("{}.sub_account is missing", path);
    }

    let (action, mut witness_bytes) = encode_raw_witness_common_fields(path, value, Some(3));
    let (entity, entity_bytes) = match action {
        SubAccountAction::Create => {
            let entity = to_latest(&format!("{}.sub_account", path), &value["sub_account"]);
            let entity_bytes = Entity::as_slice(&entity).to_vec();
            let entity: Box<dyn SubAccountMixer> = Box::new(entity);
            (entity, entity_bytes)
        }
        _ => {
            let old_sub_account_version = util::parse_json_u32(
                &format!("{}.old_sub_account_version", path),
                &value["old_sub_account_version"],
                None,
            );
            if old_sub_account_version == 1 {
                let entity = to_v1(&format!("{}.sub_account", path), &value["sub_account"]);
                let entity_bytes = Entity::as_slice(&entity).to_vec();
                let entity: Box<dyn SubAccountMixer> = Box::new(entity);
                (entity, entity_bytes)
            } else {
                let entity = to_latest(&format!("{}.sub_account", path), &value["sub_account"]);
                let entity_bytes = Entity::as_slice(&entity).to_vec();
                let entity: Box<dyn SubAccountMixer> = Box::new(entity);
                (entity, entity_bytes)
            }
        }
    };

    let key = get_smt_key_from_json(&format!("{}.sub_account", path), &value["sub_account"]);
    let (new_root, compiled_proof) = get_smt_new_root_and_proof(&action, smt_with_history, path, key, value, entity);

    encode_smt_fields(&mut witness_bytes, new_root, compiled_proof);
    encode_v3_fields(&mut witness_bytes, path, value);

    witness_bytes.extend(encoder_util::length_of(&entity_bytes));
    witness_bytes.extend(entity_bytes);

    encode_edit_fields(&action, path, &mut witness_bytes, &value);

    das_util::wrap_raw_witness_v2(DataType::SubAccount, witness_bytes)
}

fn encode_raw_witness_common_fields(
    path: &str,
    value: &Value,
    default_version: Option<u32>,
) -> (SubAccountAction, Vec<u8>) {
    let mut witness_bytes = Vec::new();

    let field_value =
        util::parse_json_u32(&format!("{}.version", path), &value["version"], default_version).to_le_bytes();
    witness_bytes.extend(encoder_util::length_of(&field_value));
    witness_bytes.extend(field_value);

    let action = SubAccountAction::from_str(value["action"].as_str().expect("witness.action should be a valid str."))
        .expect("witness.action should be a valid SubAccountAction.");

    let action_str = action.clone().to_string();
    witness_bytes.extend(encoder_util::length_of(action_str.as_bytes()));
    witness_bytes.extend(action_str.as_bytes());

    let field_value =
        util::parse_json_hex_with_default(&format!("{}.signature", path), &value["signature"], vec![255u8; 65]);
    witness_bytes.extend(encoder_util::length_of(&field_value));
    witness_bytes.extend(field_value);

    let field_value = util::parse_json_hex_with_default(&format!("{}.sign_role", path), &value["sign_role"], vec![0]);
    witness_bytes.extend(encoder_util::length_of(&field_value));
    witness_bytes.extend(field_value);

    let field_value =
        util::parse_json_u64(&format!("{}.sign_expired_at", path), &value["sign_expired_at"], Some(0)).to_le_bytes();
    witness_bytes.extend(encoder_util::length_of(&field_value));
    witness_bytes.extend(field_value);

    (action, witness_bytes)
}

fn encode_v3_fields(witness_bytes: &mut Vec<u8>, path: &str, value: &Value) {
    let old_sub_account_version = util::parse_json_u32(
        &format!("{}.old_sub_account_version", path),
        &value["old_sub_account_version"],
        None,
    );
    let field_value = old_sub_account_version.to_le_bytes();
    witness_bytes.extend(encoder_util::length_of(&field_value));
    witness_bytes.extend(field_value);

    let field_value = util::parse_json_u32(
        &format!("{}.new_sub_account_version", path),
        &value["new_sub_account_version"],
        None,
    )
    .to_le_bytes();
    witness_bytes.extend(encoder_util::length_of(&field_value));
    witness_bytes.extend(field_value);
}

fn get_smt_key_from_json(path: &str, sub_account_value: &Value) -> [u8; 32] {
    let suffix = util::parse_json_str(&format!("{}.suffix", path), &sub_account_value["suffix"]);
    let (account, _) = util::parse_json_to_account_chars(
        &format!("{}.account", path),
        &sub_account_value["account"],
        Some(suffix),
    );
    util::gen_smt_key_from_account(&account)
}

fn get_smt_new_root_and_proof(
    action: &SubAccountAction,
    smt_with_history: &mut SMTWithHistory,
    path: &str,
    key: [u8; 32],
    value: &Value,
    sub_account: Box<dyn SubAccountMixer>,
) -> ([u8; 32], Vec<u8>) {
    // Upgrade the earlier version to the latest version, because the new SubAccount should always be kept up to date.
    let sub_account = if sub_account.version() == 1 {
        let sub_account = sub_account
            .try_into_v1()
            .expect("The SubAccount should be the latest version.");

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
            .expect("The SubAccount should be the latest version.")
    };

    let sub_account = match action {
        SubAccountAction::Create => sub_account,
        SubAccountAction::Renew => {
            let expired_at = Uint64::from(util::parse_json_u64(
                "witness.edit_value.expired_at",
                &value["edit_value"]["expired_at"],
                None,
            ));
            let current_nonce = u64::from(sub_account.nonce());

            let mut builder = sub_account.as_builder();
            builder = builder.expired_at(expired_at);
            builder = builder.nonce(Uint64::from(current_nonce + 1));
            builder.build()
        }
        SubAccountAction::Edit => {
            let current_nonce = u64::from(sub_account.nonce());
            let mut builder = sub_account.clone().as_builder();
            // Modify SubAccount base on edit_key and edit_value.
            let edit_key = util::parse_json_str(&format!("{}.edit_key", path), &value["edit_key"]);
            match edit_key {
                "records" => {
                    let mol = util::parse_json_to_records_mol(&format!("{}.edit_value", path), &value["edit_value"]);
                    builder = builder.records(mol)
                }
                // WARNING The _ pattern is used to test empty edit_key so it also contains "owner" | "manager" .
                _ => {
                    let mut lock_builder = sub_account.lock().as_builder();
                    let args = util::parse_json_hex(&format!("{}.edit_value", path), &value["edit_value"]);
                    lock_builder = lock_builder.args(Bytes::from(args));

                    builder = builder.lock(lock_builder.build())
                }
            };
            builder = builder.nonce(Uint64::from(current_nonce + 1));
            builder.build()
        }
        SubAccountAction::Recycle => sub_account,
        SubAccountAction::CreateApproval | SubAccountAction::DelayApproval => {
            let current_nonce = u64::from(sub_account.nonce());
            let mut builder = Clone::clone(&sub_account).as_builder();
            let approval = encode_v2_fields(&format!("{}.edit_value", path), &value["edit_value"]);
            builder = builder.approval(approval);
            builder = builder.status(Uint8::from(AccountStatus::ApprovedTransfer as u8));
            builder = builder.nonce(Uint64::from(current_nonce + 1));
            builder.build()
        }
        SubAccountAction::RevokeApproval => {
            let current_nonce = u64::from(sub_account.nonce());
            let mut builder = Clone::clone(&sub_account).as_builder();
            builder = builder.approval(AccountApproval::default());
            builder = builder.status(Uint8::from(AccountStatus::Normal as u8));
            builder = builder.nonce(Uint64::from(current_nonce + 1));
            builder.build()
        }
        SubAccountAction::FulfillApproval => {
            let approval = sub_account.approval().clone();
            let approval_reader = approval.as_reader();
            let approval_params = approval_reader.params().raw_data();
            let approval_params_reader = AccountApprovalTransferReader::from_compatible_slice(approval_params)
                .expect("The approval params should be AccountApprovalTransferReader.");
            let current_nonce = u64::from(sub_account.nonce());

            let mut builder = Clone::clone(&sub_account).as_builder();
            builder = builder.lock(approval_params_reader.to_lock().to_entity());
            builder = builder.approval(AccountApproval::default());
            builder = builder.status(Uint8::from(AccountStatus::Normal as u8));
            builder = builder.nonce(Uint64::from(current_nonce + 1));
            builder.build()
        } // _ => unimplemented!("Not support action: {}", action),
    };

    // println!("sub_account = {}", sub_account.as_prettier());

    let smt_value = match action {
        SubAccountAction::Recycle => {
            let mut smt_value = [0u8; 32];
            // temporarily use edit_value to pass the value of SMT leaf
            let tmp =
                util::parse_json_hex_with_default(&format!("{}.edit_value", path), &value["edit_value"], vec![0u8; 32]);
            smt_value.copy_from_slice(&tmp);
            smt_value
        }
        _ => util::blake2b_smt(sub_account.as_slice().to_vec()),
    };
    let (_, new_root, proof) = smt_with_history.insert(key.clone().into(), smt_value.clone().into());
    let compiled_proof = proof.compile(vec![key.into()]).unwrap().0;

    (new_root, compiled_proof)
}

fn encode_smt_fields(witness_bytes: &mut Vec<u8>, new_root: [u8; 32], proof: Vec<u8>) {
    witness_bytes.extend(encoder_util::length_of(&new_root));
    witness_bytes.extend(new_root);

    witness_bytes.extend(encoder_util::length_of(&proof));
    witness_bytes.extend(proof);
}

fn encode_edit_fields(action: &SubAccountAction, path: &str, witness_bytes: &mut Vec<u8>, value: &Value) {
    if value["edit_key"].is_null() {
        witness_bytes.extend(encoder_util::length_of(&[]));
    } else {
        let edit_key = util::parse_json_str(&format!("{}.edit_key", path), &value["edit_key"]);
        witness_bytes.extend(encoder_util::length_of(edit_key.as_bytes()));
        witness_bytes.extend(edit_key.as_bytes().to_vec());
    }

    if value["edit_value"].is_null() {
        witness_bytes.extend(encoder_util::length_of(&[]));
    } else {
        let edit_value = match action {
            SubAccountAction::Renew => {
                let expired_at = Uint64::from(util::parse_json_u64(
                    "witness.edit_value.expired_at",
                    &value["edit_value"]["expired_at"],
                    None,
                ));
                let mut ret = expired_at.as_slice().to_vec();
                let rest =
                    util::parse_json_hex_with_default("witness.edit_value.rest", &value["edit_value"]["rest"], vec![]);
                ret.extend(rest);
                ret
            }
            SubAccountAction::Edit => {
                // Allow the edit_key field to be an invalid value.
                let edit_key = util::parse_json_str_with_default(&format!("{}.edit_key", path), &value["edit_key"], "");
                match edit_key {
                    "owner" => util::parse_json_hex(&format!("{}.edit_value", path), &value["edit_value"]),
                    "manager" => util::parse_json_hex(&format!("{}.edit_value", path), &value["edit_value"]),
                    "records" => {
                        let mol =
                            util::parse_json_to_records_mol(&format!("{}.edit_value", path), &value["edit_value"]);
                        mol.as_slice().to_vec()
                    }
                    // If the edit_key field is invalid just parse edit_value field as hex string.
                    _ => util::parse_json_hex(&format!("{}.edit_value", path), &value["edit_value"]),
                }
            }
            SubAccountAction::CreateApproval | SubAccountAction::DelayApproval => {
                let approval = encode_v2_fields(&format!("{}.edit_value", path), &value["edit_value"]);
                approval.as_slice().to_vec()
            }
            SubAccountAction::RevokeApproval | SubAccountAction::FulfillApproval => {
                if !value["edit_value"].is_null() {
                    // This should not happen, but we still need to build the transaction with the error to test it.
                    let approval = encode_v2_fields(&format!("{}.edit_value", path), &value["edit_value"]);
                    approval.as_slice().to_vec()
                } else {
                    vec![]
                }
            }
            _ => util::parse_json_hex(&format!("{}.edit_value", path), &value["edit_value"]),
        };

        witness_bytes.extend(encoder_util::length_of(&edit_value));
        witness_bytes.extend(edit_value);
    }
}
