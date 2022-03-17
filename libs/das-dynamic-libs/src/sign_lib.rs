use super::util;
use alloc::vec::Vec;
use ckb_std::dynamic_loading_c_impl::{CKBDLContext, Symbol};

// int validate(int type, uint8_t* message, uint8_t* lock_bytes, uint8_t* eth_address)
type ValidateFunction =
    unsafe extern "C" fn(type_no: i32, message: *const u8, lock_bytes: *const u8, lock_args: *const u8) -> i32;
type ValidateStrFunction = unsafe extern "C" fn(
    type_no: i32,
    message: *const u8,
    message_len: usize,
    lock_bytes: *const u8,
    lock_args: *const u8,
) -> i32;

pub struct SignLib {
    c_validate: Symbol<ValidateFunction>,
    c_validate_str: Symbol<ValidateStrFunction>,
}

impl SignLib {
    /// Load signature validation libraries from das-lock
    ///
    /// Required memory size: about 128 * 1024
    pub fn load<T>(context: &mut CKBDLContext<T>, code_hash: &[u8]) -> Self {
        let lib = context
            .load(code_hash)
            .expect("The shared lib should be loaded successfully.");

        SignLib {
            c_validate: unsafe {
                lib.get(b"validate")
                    .expect("Load function 'validate' from library failed.")
            },
            c_validate_str: unsafe {
                lib.get(b"validate_str")
                    .expect("Load function 'validate_str' from library failed.")
            },
        }
    }

    /// Validate signatures
    ///
    /// costs: about 2_000_000 cycles
    pub fn validate(&self, type_no: i32, digest: Vec<u8>, lock_bytes: Vec<u8>, lock_args: Vec<u8>) -> Result<(), i32> {
        let func = &self.c_validate;
        let error_code: i32 = unsafe { func(type_no, digest.as_ptr(), lock_bytes.as_ptr(), lock_args.as_ptr()) };
        if error_code != 0 {
            return Err(error_code);
        }

        Ok(())
    }

    pub fn validate_str(
        &self,
        type_no: i32,
        digest: Vec<u8>,
        digest_len: usize,
        lock_bytes: Vec<u8>,
        lock_args: Vec<u8>,
    ) -> Result<(), i32> {
        let func = &self.c_validate_str;
        let error_code: i32 = unsafe {
            func(
                type_no,
                digest.as_ptr(),
                digest_len,
                lock_bytes.as_ptr(),
                lock_args.as_ptr(),
            )
        };
        if error_code != 0 {
            return Err(error_code);
        }

        Ok(())
    }

    pub fn gen_digest(&self, edit_key: Vec<u8>, edit_value: Vec<u8>, nonce: Vec<u8>) -> Vec<u8> {
        let mut blake2b = util::new_blake2b();
        blake2b.update(&edit_key);
        blake2b.update(&edit_value);
        blake2b.update(&nonce);
        let mut h = [0u8; 32];
        blake2b.finalize(&mut h);
        let s = "from did: ";
        let mut message = s.as_bytes().to_vec();
        message.append(&mut h.to_vec());
        message
    }

    pub fn verify_sub_account_sig(
        &self,
        edit_key: Vec<u8>,
        edit_value: Vec<u8>,
        nonce: Vec<u8>,
        sig: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<(), i32> {
        let message = self.gen_digest(edit_key, edit_value, nonce);
        let type_no = 0i32;
        let m_len = message.len();
        let ret = self.validate_str(type_no, message, m_len, sig, args);
        if let Err(error_code) = ret {
            return Err(error_code);
        } else {
            Ok(())
        }
    }
}
