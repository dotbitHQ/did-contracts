use alloc::vec::Vec;
use ckb_std::dynamic_loading_c_impl::{CKBDLContext, Symbol};

// int validate(int type, uint8_t* message, uint8_t* lock_bytes, uint8_t* eth_address)
type ValidateFunction =
    unsafe extern "C" fn(type_no: i32, message: *const u8, lock_bytes: *const u8, lock_args: *const u8) -> i32;

pub struct SignLib {
    c_validate: Symbol<ValidateFunction>,
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
}
