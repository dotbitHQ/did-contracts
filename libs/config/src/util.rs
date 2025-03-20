use das_types::constants::ACCOUNT_ID_LENGTH;
use das_types::util::blake2b_256;

pub fn hash_account(account_without_suffix: &[u8]) -> [u8; ACCOUNT_ID_LENGTH] {
    let account_full_hash = blake2b_256(account_without_suffix);

    let mut account_hash = [0u8; ACCOUNT_ID_LENGTH];
    account_hash.copy_from_slice(&account_full_hash[0..ACCOUNT_ID_LENGTH]);
    account_hash
}

pub fn is_account_in_collection(account_hash: &[u8], collection: &[u8]) -> bool {
    let length = collection.len();
    if length <= 0 {
        return false;
    }

    let first = &collection[0..20];
    let last = &collection[length - 20..];

    return if account_hash < first {
        // debug!("The account is less than the first preserved account, skip.");
        false
    } else if account_hash > last {
        // debug!("The account is bigger than the last preserved account, skip.");
        false
    } else {
        let accounts_total = collection.len() / ACCOUNT_ID_LENGTH;
        let mut start_account_index = 0;
        let mut end_account_index = accounts_total - 1;

        loop {
            let mid_account_index = (start_account_index + end_account_index) / 2;
            // debug!("mid_account_index = {:?}", mid_account_index);
            let mid_account_start_byte_index = mid_account_index * ACCOUNT_ID_LENGTH;
            let mid_account_end_byte_index = mid_account_start_byte_index + ACCOUNT_ID_LENGTH;
            let mid_account_bytes = collection
                .get(mid_account_start_byte_index..mid_account_end_byte_index)
                .unwrap();

            if mid_account_bytes < account_hash {
                start_account_index = mid_account_index + 1;
                // debug!("<");
            } else if mid_account_bytes > account_hash {
                // debug!(">");
                end_account_index = if mid_account_index > 1 {
                    mid_account_index - 1
                } else {
                    0
                };
            } else {
                return true;
            }

            if start_account_index > end_account_index || end_account_index == 0 {
                break;
            }
        }

        false
    };
}
