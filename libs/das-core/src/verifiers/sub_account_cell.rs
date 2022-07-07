use crate::{
    assert,
    constants::*,
    data_parser, debug,
    error::Error,
    sub_account_witness_parser::{SubAccountEditValue, SubAccountWitness},
    util, warn,
    witness_parser::WitnessesParser,
};
use alloc::string::String;
use das_dynamic_libs::{error::Error as DasDynamicLibError, sign_lib::SignLib};
use das_types::{constants::*, packed::*, prelude::Entity, prettier::Prettier};
use sparse_merkle_tree::{ckb_smt::SMTBuilder, H256};

pub fn verify_unlock_role(witness: &SubAccountWitness) -> Result<(), Error> {
    debug!(
        "witnesses[{}] Verify if the witness is unlocked by expected role.",
        witness.index
    );

    let required_role = match witness.edit_value {
        SubAccountEditValue::Records(_) => LockRole::Manager,
        _ => LockRole::Owner,
    };

    assert!(
        witness.sign_role == Some(required_role),
        Error::AccountCellPermissionDenied,
        "witnesses[{}] This witness should be unlocked by the {:?}'s signature.",
        witness.index,
        required_role
    );

    Ok(())
}

pub fn verify_status(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    expected_status: AccountStatus,
) -> Result<(), Error> {
    debug!(
        "witnesses[{}] Verify if the witness.sub_account.status is not expected.",
        sub_account_index
    );

    let sub_account_status = u8::from(sub_account_reader.status());

    debug!(
        "witnesses[{}] The witness.sub_account.status of {} should be {:?}.",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader),
        expected_status
    );

    assert!(
        sub_account_status == expected_status as u8,
        Error::AccountCellStatusLocked,
        "witnesses[{}] The witness.sub_account.status of {} should be {:?}.",
        sub_account_index,
        sub_account_reader.account().as_prettier(),
        expected_status
    );

    Ok(())
}

pub fn verify_expiration(
    config: ConfigCellAccountReader,
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    current: u64,
) -> Result<(), Error> {
    debug!(
        "witnesses[{}] Verify if the witness.sub_account.expired_at of sub-account is expired.",
        sub_account_index
    );

    let expired_at = u64::from(sub_account_reader.expired_at());
    let expiration_grace_period = u32::from(config.expiration_grace_period()) as u64;

    if current > expired_at {
        if current - expired_at > expiration_grace_period {
            warn!(
                "witnesses[{}] The sub-account {} has been expired. Will be recycled soon.",
                sub_account_index,
                sub_account_reader.account().as_prettier()
            );
            return Err(Error::AccountCellHasExpired);
        } else {
            warn!("witnesses[{}] The sub-account {} has been in expiration grace period. Need to be renew as soon as possible.", sub_account_index, sub_account_reader.account().as_prettier());
            return Err(Error::AccountCellInExpirationGracePeriod);
        }
    }

    Ok(())
}

pub fn verify_suffix_with_parent_account(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    parent_account: &[u8],
) -> Result<(), Error> {
    debug!(
        "witnesses[{}] Verify if the witness.sub_account is child of the AccountCell in transaction.",
        sub_account_index
    );

    let mut expected_suffix = b".".to_vec();
    expected_suffix.extend(parent_account);

    let suffix = sub_account_reader.suffix().raw_data();

    assert!(
        expected_suffix == suffix,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.suffix of {} should come from the parent account.(expected: {:?}, current: {:?})",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader),
        String::from_utf8(expected_suffix),
        String::from_utf8(suffix.to_vec())
    );

    Ok(())
}

fn verify_initial_lock(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
    let expected_lock = das_lock();
    let current_lock = sub_account_reader.lock();

    assert!(
        util::is_type_id_equal(expected_lock.as_reader(), current_lock.into()),
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.lock of {} must be a das-lock.",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader)
    );

    data_parser::das_lock_args::get_owner_and_manager(current_lock.args().raw_data())?;

    Ok(())
}

fn verify_initial_id(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
    let account = util::get_sub_account_name_from_reader(sub_account_reader);
    let expected_account_id = util::get_account_id_from_account(account.as_bytes());
    let account_id = sub_account_reader.id().raw_data();

    assert!(
        &expected_account_id == account_id,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.id of {} do not match.(expected: 0x{}, current: 0x{})",
        sub_account_index,
        account,
        util::hex_string(&expected_account_id),
        util::hex_string(account_id)
    );

    Ok(())
}

fn verify_initial_registered_at(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    timestamp: u64,
) -> Result<(), Error> {
    let registered_at = u64::from(sub_account_reader.registered_at());

    assert!(
        registered_at == timestamp,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.registered_at of {} should be the same as the timestamp in TimeCell.(expected: {}, current: {})",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader),
        timestamp,
        registered_at
    );

    Ok(())
}

pub fn verify_initial_properties(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    current_timestamp: u64,
) -> Result<(), Error> {
    debug!(
        "witnesses[{}] Verify if the initial properties of sub-account is filled properly.",
        sub_account_index
    );

    verify_initial_lock(sub_account_index, sub_account_reader)?;
    verify_initial_id(sub_account_index, sub_account_reader)?;
    verify_initial_registered_at(sub_account_index, sub_account_reader, current_timestamp)?;
    verify_status(sub_account_index, sub_account_reader, AccountStatus::Normal)?;

    assert!(
        sub_account_reader.records().len() == 0,
        Error::AccountCellRecordNotEmpty,
        "witnesses[{}] The witness.sub_account.records of {} should be empty.",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader)
    );

    let enable_sub_account = u8::from(sub_account_reader.enable_sub_account());
    assert!(
        enable_sub_account == 0,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.enable_sub_account of {} should be 0 .",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader)
    );

    let renew_sub_account_price = u64::from(sub_account_reader.renew_sub_account_price());
    assert!(
        renew_sub_account_price == 0,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.renew_sub_account_price of {} should be 0 .",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader)
    );

    let nonce = u64::from(sub_account_reader.nonce());
    assert!(
        nonce == 0,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.nonce of {} should be 0 .",
        sub_account_index,
        util::get_sub_account_name_from_reader(sub_account_reader)
    );

    let expired_at = u64::from(sub_account_reader.expired_at());
    assert!(
        expired_at >= current_timestamp + YEAR_SEC,
        Error::SubAccountInitialValueError,
        "witnesses[{}] The witness.sub_account.expired_at should be at least one year.(expected: >= {}, current: {})",
        sub_account_index,
        current_timestamp + YEAR_SEC,
        expired_at
    );

    Ok(())
}

pub fn verify_smt_proof(key: [u8; 32], val: [u8; 32], root: [u8; 32], proof: &[u8]) -> Result<(), Error> {
    if cfg!(feature = "dev") {
        // CAREFUL Proof verification has been skipped in development mode.
        return Ok(());
    }

    let builder = SMTBuilder::new();
    let builder = builder.insert(&H256::from(key), &H256::from(val)).unwrap();

    let smt = builder.build().unwrap();
    let ret = smt.verify(&H256::from(root), &proof);
    if let Err(_e) = ret {
        debug!("verify_smt_proof verification failed. Err: {:?}", _e);
        return Err(Error::SubAccountWitnessSMTRootError);
    } else {
        debug!("verify_smt_proof verification passed.");
    }
    Ok(())
}

pub fn verify_sub_account_sig(witness: &SubAccountWitness, sign_lib: &SignLib) -> Result<(), Error> {
    if cfg!(feature = "dev") {
        // CAREFUL Proof verification has been skipped in development mode.
        debug!(
            "witnesses[{}] Skip verifying the witness.sub_account.sig is valid.",
            witness.index
        );
        return Ok(());
    }

    debug!(
        "witnesses[{}] Verify if the witness.sub_account.sig is valid.",
        witness.index
    );

    let das_lock_type = match witness.sign_type {
        Some(val) if val == DasLockType::ETH || val == DasLockType::ETHTypedData || val == DasLockType::TRON => val,
        _ => {
            warn!(
                "witnesses[{}] Parsing das-lock(witness.sub_account.lock.args) algorithm failed (maybe not supported for now), but it is required in this transaction.",
                witness.index
            );
            return Err(Error::InvalidTransactionStructure);
        }
    };

    let account_id = witness.sub_account.id().as_slice().to_vec();
    let edit_key = witness.edit_key.as_slice();
    let edit_value = witness.edit_value_bytes.as_slice();
    let nonce = witness.sub_account.nonce().as_slice().to_vec();
    let signature = witness.signature.as_slice();
    let args = witness.sign_args.as_slice();

    let ret = sign_lib.verify_sub_account_sig(
        das_lock_type,
        account_id,
        edit_key.to_vec(),
        edit_value.to_vec(),
        nonce,
        signature.to_vec(),
        args.to_vec(),
    );

    match ret {
        Err(_error_code) if _error_code == DasDynamicLibError::UndefinedDasLockType as i32 => {
            warn!(
                "witnesses[{}] The signature algorithm has not been supported",
                witness.index
            );
            Err(Error::HardCodedError)
        }
        Err(_error_code) => {
            warn!(
                "witnesses[{}] The witness.signature is invalid, the error_code returned by dynamic library is: {}",
                witness.index, _error_code
            );
            Err(Error::SubAccountSigVerifyError)
        }
        _ => {
            debug!("witnesses[{}] The witness.signature is valid.", witness.index);
            Ok(())
        }
    }
}

const SUB_ACCOUNT_BETA_LIST_WILDCARD: [u8; 20] = [
    216, 59, 196, 4, 163, 94, 224, 196, 194, 5, 93, 90, 193, 58, 92, 50, 58, 174, 73, 74,
];

/// Verify if the account can join sub-account feature beta.
pub fn verify_beta_list(parser: &WitnessesParser, account: &[u8]) -> Result<(), Error> {
    debug!("Verify if the account can join sub-account feature beta");

    let account_hash = util::blake2b_256(account);
    let account_id = account_hash.get(..ACCOUNT_ID_LENGTH).unwrap();
    let sub_account_beta_list = parser.configs.sub_account_beta_list()?;

    if sub_account_beta_list == &SUB_ACCOUNT_BETA_LIST_WILDCARD {
        debug!("The wildcard '*' of beta list is matched.");
        return Ok(());
    } else if !util::is_account_id_in_collection(account_id, sub_account_beta_list) {
        warn!(
            "The account is not allow to enable sub-account feature in beta test.(account: {}, account_id: 0x{})",
            String::from_utf8(account.to_vec()).unwrap(),
            util::hex_string(account_id)
        );
        return Err(Error::SubAccountJoinBetaError);
    }

    debug!(
        "Found account {:?} in the beta list.",
        String::from_utf8(account.to_vec())
    );

    Ok(())
}
