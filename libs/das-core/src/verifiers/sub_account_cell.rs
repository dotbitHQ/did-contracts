use crate::{assert, constants::*, debug, error::Error, warn, witness_parser::WitnessesParser};
use alloc::vec::Vec;
use das_types::{constants::*, packed::*, prettier::Prettier};
use sparse_merkle_tree::{
    ckb_smt::SMTBuilder,
    H256
};
use ckb_std::dynamic_loading_c_impl::CKBDLContext;
use das_dynamic_libs::sign_lib::SignLib;

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

pub fn verify_lock(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
    Ok(())
}

pub fn verify_id(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
    Ok(())
}

pub fn verify_suffix(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    parent_account: &[u8],
) -> Result<(), Error> {
    Ok(())
}

pub fn verify_expired_at(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    timestamp: u64,
) -> Result<(), Error> {
    Ok(())
}

pub fn verify_registered_at(
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
        sub_account_reader.account().as_prettier(),
        timestamp,
        registered_at
    );

    Ok(())
}

pub fn verify_record_empty(sub_account_index: usize, sub_account_reader: SubAccountReader) -> Result<(), Error> {
    debug!("Check if witness.sub_account.records of sub-account is empty.");

    assert!(
        sub_account_reader.records().len() == 0,
        Error::AccountCellRecordNotEmpty,
        "witnesses[{}] The witness.sub_account.records of {} should be empty.",
        sub_account_index,
        sub_account_reader
    );

    Ok(())
}

pub fn verify_status(
    sub_account_index: usize,
    sub_account_reader: SubAccountReader,
    expected_status: AccountStatus,
) -> Result<(), Error> {
    debug!("Verify if witness.sub_account.status of sub-account is not expected.");

    let sub_account_status = u8::from(sub_account_reader.status());

    assert!(
        sub_account_status == expected_status as u8,
        Error::AccountCellStatusLocked,
        "witnesses[{}] The witness.sub_account.status of {} should be {:?}.",
        sub_account_index,
        sub_account_reader,
        expected_status
    );

    Ok(())
}

pub fn verify_records_keys(
    parser: &WitnessesParser,
    record_key_namespace: &Vec<u8>,
    records: RecordsReader,
) -> Result<(), Error> {
    let config = parser.configs.account()?;
    let records_max_size = u32::from(config.record_size_limit()) as usize;

    assert!(
        records.total_size() <= records_max_size,
        Error::AccountCellRecordSizeTooLarge,
        "The total size of all records can not be more than {} bytes.",
        records_max_size
    );

    // extract all the keys, which are split by 0
    let mut key_start_at = 0;
    let mut key_list = Vec::new();
    for (index, item) in record_key_namespace.iter().enumerate() {
        if *item == 0 {
            let key_vec = &record_key_namespace[key_start_at..index];
            key_start_at = index + 1;

            key_list.push(key_vec);
        }
    }

    fn vec_compare(va: &[u8], vb: &[u8]) -> bool {
        // zip stops at the shortest
        (va.len() == vb.len()) && va.iter().zip(vb).all(|(a, b)| a == b)
    }

    // check if all the record.{type+key} are valid
    for record in records.iter() {
        let mut is_valid = false;

        let mut record_type = Vec::from(record.record_type().raw_data());
        let mut record_key = Vec::from(record.record_key().raw_data());
        if record_type == b"custom_key" {
            // CAREFUL Triple check
            for char in record_key.iter() {
                assert!(
                    CUSTOM_KEYS_NAMESPACE.contains(char),
                    Error::AccountCellRecordKeyInvalid,
                    "The keys in custom_key should only contain digits, lowercase alphabet and underline."
                );
            }
            continue;
        }

        record_type.push(46);
        record_type.append(&mut record_key);

        for key in &key_list {
            if vec_compare(record_type.as_slice(), *key) {
                is_valid = true;
                break;
            }
        }

        if !is_valid {
            assert!(
                false,
                Error::AccountCellRecordKeyInvalid,
                "Account cell record key is invalid: {:?}", record_type
            );

            break;
        }
    }

    Ok(())
}


pub fn verify_smt_proof(key: [u8; 32], val: [u8; 32], root: [u8; 32], proof: &[u8]) -> Result<(), Error> {
    let builder = SMTBuilder::new();
    let builder = builder.insert(&H256::from(key), &H256::from(val)).unwrap();

    let smt = builder.build().unwrap();
    let ret = smt.verify(&H256::from(root), &proof);
    if let Err(e) = ret {
        debug!("verify_smt_proof verification failed. Err: {:?}", e);
        return Err(Error::SubAccountWitnessSMTRootError);
    } else {
        debug!("verify_smt_proof verification passed.");
    }
    Ok(())
}

pub fn verify_sub_account_sig(edit_key: &[u8], edit_value: &[u8], nonce: &[u8], sig: &[u8], args: &[u8]) -> Result<(), Error> {
    let mut context = unsafe { CKBDLContext::<[u8; 128 * 1024]>::new() };
    // TODO: need to be used as a param
    #[cfg(feature = "mainnet")]
    let code_hash: [u8; 32] = [
        114,136,18,7,241,131,151,251,114,137,71,94,28,208,216,64,104,55,4,5,126,140,166,6,43,114,139,209,174,122,155,68
    ];
    #[cfg(not(feature = "mainnet"))]
    let code_hash: [u8; 32] = [
        114,136,18,7,241,131,151,251,114,137,71,94,28,208,216,64,104,55,4,5,126,140,166,6,43,114,139,209,174,122,155,68
    ];

    let lib = SignLib::load(&mut context, &code_hash);
    let ret = lib.verify_sub_account_sig(edit_key.to_vec(), edit_value.to_vec(), nonce.to_vec(), sig.to_vec(), args.to_vec());
    if let Err(error_code) = ret {
        debug!("verify_sub_account_sig failed, error_code: {}", error_code);
        return Err(Error::SubAccountSigVerifyError);
    } else {
        debug!("verify_sub_account_sig succeed.");
    }
    Ok(())
}