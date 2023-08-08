use alloc::string::ToString;
use alloc::vec::Vec;

use ckb_std::dynamic_loading_c_impl::Symbol;
use das_types::constants::{DasLockType, SubAccountAction};
use das_types::packed::*;

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

type ValidateDeviceFunction = unsafe extern "C" fn(
    version: i32,
    sig: *const u8,
    sig_len: usize,
    msg: *const u8,
    msg_len: usize,
    device_key_list: *const u8,
    device_key_list_len: usize,
    data: *const u8,
    data_len: usize,
) -> i32;

pub struct SignLibWith3Methods {
    pub c_validate: Symbol<ValidateFunction>,
    pub c_validate_str: Symbol<ValidateStrFunction>,
    pub c_validate_device: Symbol<ValidateDeviceFunction>,
}

pub struct SignLibWith2Methods {
    pub c_validate: Symbol<ValidateFunction>,
    pub c_validate_str: Symbol<ValidateStrFunction>,
}

pub struct SignLibWith1Methods {
    pub c_validate: Symbol<ValidateFunction>,
}

pub struct SignLib {
    pub ckb_signhash: Option<SignLibWith1Methods>,
    pub ckb_multisig: Option<SignLibWith1Methods>,
    pub ed25519: Option<SignLibWith2Methods>,
    pub eth: Option<SignLibWith2Methods>,
    pub tron: Option<SignLibWith2Methods>,
    pub doge: Option<SignLibWith2Methods>,
    pub web_authn: Option<SignLibWith3Methods>,
}

impl SignLib {
    pub fn new() -> Self {
        SignLib {
            ckb_signhash: None,
            ckb_multisig: None,
            ed25519: None,
            eth: None,
            tron: None,
            doge: None,
            web_authn: None,
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

        let func;

        match das_lock_type {
            DasLockType::CKBSingle => {
                let lib = self.ckb_signhash.as_ref().unwrap();
                func = &lib.c_validate;
            }
            DasLockType::CKBMulti => {
                let lib = self.ckb_multisig.as_ref().unwrap();
                func = &lib.c_validate;
            }
            DasLockType::ETH | DasLockType::ETHTypedData => {
                let lib = self.eth.as_ref().unwrap();
                func = &lib.c_validate;
            }
            DasLockType::TRON => {
                let lib = self.tron.as_ref().unwrap();
                func = &lib.c_validate;
            }
            DasLockType::Doge => {
                let lib = self.doge.as_ref().unwrap();
                func = &lib.c_validate;
            }
            _ => return Err(Error::UndefinedDasLockType as i32),
        }

        let error_code: i32 = unsafe { func(type_no, digest.as_ptr(), lock_bytes.as_ptr(), lock_args.as_ptr()) };

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

        let func;
        match das_lock_type {
            // DasLockType::CKBSingle => {
            //     let lib = self.ckb_signhash.as_ref().unwrap();
            //     func = &lib.c_validate_str;
            // }
            DasLockType::ETH | DasLockType::ETHTypedData => {
                let lib = self.eth.as_ref().unwrap();
                func = &lib.c_validate_str;
            }
            DasLockType::TRON => {
                let lib = self.tron.as_ref().unwrap();
                func = &lib.c_validate_str;
            }
            DasLockType::Doge => {
                let lib = self.doge.as_ref().unwrap();
                func = &lib.c_validate_str;
            }
            DasLockType::WebAuthn => {
                let lib = self.web_authn.as_ref().unwrap();
                func = &lib.c_validate_str;
            }
            _ => return Err(Error::UndefinedDasLockType as i32),
        }

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

    pub fn validate_device(
        &self,
        das_lock_type: DasLockType,
        version: i32,
        sig: &[u8],
        msg: &[u8],
        device_key_list: &[u8],
        data: &[u8],
    ) -> Result<(), i32> {
        if cfg!(feature = "dev") {
            return Ok(());
        }

        if das_lock_type != DasLockType::WebAuthn {
            return Err(Error::UndefinedDasLockType as i32);
        }

        warn_log!(
            "SignLib::validate_device The params pass to dynamic lib is {{ version: {}, sig: 0x{}, msg: 0x{}, device_key_list: 0x{}, data: 0x{} }}",
            version,
            util::hex_string(sig),
            util::hex_string(msg),
            util::hex_string(device_key_list),
            util::hex_string(data)
        );

        let func = &self.web_authn.as_ref().unwrap().c_validate_device;

        let error_code = unsafe {
            func(
                version,
                sig.as_ptr(),
                sig.len(),
                msg.as_ptr(),
                msg.len(),
                device_key_list.as_ptr(),
                device_key_list.len(),
                data.as_ptr(),
                data.len(),
            )
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
        if cfg!(feature = "dev") {
            return Ok(());
        }

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

    pub fn verify_sub_account_approval_sig(
        &self,
        das_lock_type: DasLockType,
        action: SubAccountAction,
        approval: AccountApprovalReader,
        nonce: Vec<u8>,
        sig: Vec<u8>,
        args: Vec<u8>,
        sign_expired_at: Vec<u8>,
    ) -> Result<(), i32> {
        if cfg!(feature = "dev") {
            return Ok(());
        }

        let action_bytes = action.to_string().as_bytes().to_vec();
        let approval_bytes = approval.as_slice().to_vec();
        let data = [action_bytes, approval_bytes, nonce, sign_expired_at].concat();
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
        if cfg!(feature = "dev") {
            return Ok(());
        }

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

    pub fn gen_digest(&self, das_lock_type: DasLockType, data: Vec<u8>) -> Result<Vec<u8>, i32> {
        let mut blake2b = util::new_blake2b();
        blake2b.update(&data);
        let mut h = [0u8; 32];
        blake2b.finalize(&mut h);

        match das_lock_type {
            // DasLockType::ETHTypedData => {
            //     let prefix = "from did: ".as_bytes();
            //     Ok([prefix, &h].concat())
            // }
            DasLockType::ETH
            | DasLockType::ETHTypedData
            | DasLockType::TRON
            | DasLockType::Doge
            | DasLockType::WebAuthn => Ok(h.to_vec()),
            _ => Err(Error::UndefinedDasLockType as i32),
        }
    }
}
