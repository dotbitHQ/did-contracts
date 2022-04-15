use crate::{assert, constants::*, data_parser, debug, error::Error, util, warn, witness_parser::WitnessesParser};
use alloc::string::String;
use das_dynamic_libs::sign_lib::SignLib;
use das_types::{constants::*, packed::*, prettier::Prettier};
use sparse_merkle_tree::{ckb_smt::SMTBuilder, H256};

pub fn verify_expiration(
    config: ConfigCellAccountReader,
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    current: u64,
) -> Result<(), Error> {
    debug!("Verify if witness.sub_account.expired_at of sub-account is expired.");

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

pub fn verify_initial_lock(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
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

pub fn verify_initial_id(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
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

pub fn verify_suffix_with_parent_account(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    parent_account: &[u8],
) -> Result<(), Error> {
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

pub fn verify_initial_registered_at(
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

pub fn verify_status(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    expected_status: AccountStatus,
) -> Result<(), Error> {
    debug!("Verify if witness.sub_account.status is not expected.");

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

pub fn verify_sub_account_sig(
    sign_lib: &SignLib,
    alg_id: i8,
    account_id: &[u8],
    edit_key: &[u8],
    edit_value: &[u8],
    nonce: &[u8],
    signature: &[u8],
    args: &[u8],
) -> Result<(), Error> {
    if cfg!(feature = "dev") {
        // CAREFUL Proof verification has been skipped in development mode.
        return Ok(());
    }
    if alg_id != 3 || alg_id != 5 {
        return Err(SubAccountSigVerifyError);
    }
    let ret = sign_lib.verify_sub_account_sig(
        alg_id,
        account_id.to_vec(),
        edit_key.to_vec(),
        edit_value.to_vec(),
        nonce.to_vec(),
        signature.to_vec(),
        args.to_vec(),
    );
    if let Err(_error_code) = ret {
        debug!("verify_sub_account_sig failed, error_code: {}", _error_code);
        return Err(Error::SubAccountSigVerifyError);
    } else {
        debug!("verify_sub_account_sig succeed.");
    }
    Ok(())
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
