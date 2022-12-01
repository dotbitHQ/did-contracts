use alloc::vec::Vec;

use ckb_std::dynamic_loading_c_impl::Symbol;

use super::constants::*;
use super::error::Error;
use super::util;

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

pub struct SignLibWith2Methods {
    pub c_validate: Symbol<ValidateFunction>,
    pub c_validate_str: Symbol<ValidateStrFunction>,
}

pub struct SignLibWith1Methods {
    pub c_validate: Symbol<ValidateFunction>,
}

pub struct SignLib {
    eth: Option<SignLibWith2Methods>,
    tron: Option<SignLibWith2Methods>,
    multi: Option<SignLibWith1Methods>,
}

impl SignLib {
    pub fn new(
        eth: Option<SignLibWith2Methods>,
        tron: Option<SignLibWith2Methods>,
        multi: Option<SignLibWith1Methods>,
    ) -> Self {
        SignLib {
            // ckb_sign_hash_all: OnceCell::new(),
            // ckb_multi_sig_all: OnceCell::new(),
            eth,
            tron,
            multi,
        }
    }

    /// Validate signatures
    ///
    /// costs: about 2_000_000 cycles
    pub fn validate(
        &self,
        das_lock_type: DasLockType,
        type_no: i32,
        digest: Vec<u8>,
        lock_bytes: Vec<u8>,
        lock_args: Vec<u8>,
    ) -> Result<(), i32> {
        warn_log!(
            "SignLib::validate The params pass to dynamic lib is {{ type_no: {}, digest: 0x{}, lock_bytes: 0x{}, lock_args: 0x{} }}",
            type_no,
            util::hex_string(&digest),
            util::hex_string(&lock_bytes),
            util::hex_string(&lock_args)
        );

        let error_code: i32 = match das_lock_type {
            DasLockType::ETH | DasLockType::ETHTypedData => {
                let lib = self.eth.as_ref().unwrap();
                let func = &lib.c_validate;
                unsafe { func(type_no, digest.as_ptr(), lock_bytes.as_ptr(), lock_args.as_ptr()) }
            }
            DasLockType::TRON => {
                let lib = self.tron.as_ref().unwrap();
                let func = &lib.c_validate;
                unsafe { func(type_no, digest.as_ptr(), lock_bytes.as_ptr(), lock_args.as_ptr()) }
            }
            DasLockType::CKBMulti => {
                let lib = self.multi.as_ref().unwrap();
                let func = &lib.c_validate;
                unsafe { func(type_no, digest.as_ptr(), lock_bytes.as_ptr(), lock_args.as_ptr()) }
            }
            _ => return Err(Error::UndefinedDasLockType as i32),
        };

        if error_code != 0 {
            return Err(error_code);
        }

        Ok(())
    }

    pub fn validate_str(
        &self,
        das_lock_type: DasLockType,
        type_no: i32,
        digest: Vec<u8>,
        digest_len: usize,
        lock_bytes: Vec<u8>,
        lock_args: Vec<u8>,
    ) -> Result<(), i32> {
        warn_log!("SignLib::validate_str das_lock_type: {:?}", das_lock_type);

        warn_log!(
            "SignLib::validate_str The params pass to dynamic lib is {{ type_no: {}, digest: 0x{}, digest_len: {}, lock_bytes: 0x{}, lock_args: 0x{} }}",
            type_no,
            util::hex_string(&digest),
            digest_len,
            util::hex_string(&lock_bytes),
            util::hex_string(&lock_args)
        );

        let error_code: i32 = match das_lock_type {
            DasLockType::ETH | DasLockType::ETHTypedData => {
                let lib = self.eth.as_ref().unwrap();
                let func = &lib.c_validate_str;
                unsafe {
                    func(
                        type_no,
                        digest.as_ptr(),
                        digest_len,
                        lock_bytes.as_ptr(),
                        lock_args.as_ptr(),
                    )
                }
            }
            DasLockType::TRON => {
                let lib = self.tron.as_ref().unwrap();
                let func = &lib.c_validate_str;
                unsafe {
                    func(
                        type_no,
                        digest.as_ptr(),
                        digest_len,
                        lock_bytes.as_ptr(),
                        lock_args.as_ptr(),
                    )
                }
            }
            _ => return Err(Error::UndefinedDasLockType as i32),
        };

        if error_code != 0 {
            return Err(error_code);
        }

        Ok(())
    }

    // TODO abstrate the common code of verify_* functions
    pub fn verify_sub_account_mint_sig(
        &self,
        das_lock_type: DasLockType,
        expired_at: Vec<u8>,
        account_list_smt_root: Vec<u8>,
        sig: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<(), i32> {
        let data = [expired_at, account_list_smt_root].concat();
        let message = self.gen_digest(das_lock_type, data)?;
        let type_no = 0i32;
        let m_len = message.len();
        let ret = self.validate_str(das_lock_type, type_no, message, m_len, sig, args);
        if let Err(error_code) = ret {
            Err(error_code)
        } else {
            Ok(())
        }
    }

    pub fn verify_sub_account_sig(
        &self,
        das_lock_type: DasLockType,
        account_id: Vec<u8>,
        edit_key: Vec<u8>,
        edit_value: Vec<u8>,
        nonce: Vec<u8>,
        sig: Vec<u8>,
        args: Vec<u8>,
        sign_expired_at: Vec<u8>,
    ) -> Result<(), i32> {
        let data = [account_id, edit_key, edit_value, nonce, sign_expired_at].concat();
        let message = self.gen_digest(das_lock_type, data)?;
        let type_no = 0i32;
        let m_len = message.len();
        let ret = self.validate_str(das_lock_type, type_no, message, m_len, sig, args);
        if let Err(error_code) = ret {
            Err(error_code)
        } else {
            Ok(())
        }
    }

    fn gen_digest(&self, das_lock_type: DasLockType, data: Vec<u8>) -> Result<Vec<u8>, i32> {
        let mut blake2b = util::new_blake2b();
        blake2b.update(&data);
        let mut h = [0u8; 32];
        blake2b.finalize(&mut h);

        match das_lock_type {
            DasLockType::ETH | DasLockType::ETHTypedData | DasLockType::TRON => {
                let prefix = "from did: ".as_bytes();
                Ok([prefix, &h].concat())
            }
            _ => Err(Error::UndefinedDasLockType as i32),
        }
    }
}
