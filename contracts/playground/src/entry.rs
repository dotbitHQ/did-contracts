use core::result::Result;
use das_core::{assert, debug, error::Error, warn};
use alloc::vec::Vec;
use ckb_std::dynamic_loading::{CKBDLContext, Symbol};


fn dynamic_loading() -> Result<(), Error> {
    use ckb_std::dynamic_loading_c_impl::CKBDLContext;
    use das_dynamic_libs::sign_lib::SignLib;

    // ckb_sign.so
    // let code_hash: [u8; 32] = [
    //     79, 105, 229, 203, 50, 2, 35, 246, 194, 141, 49, 171, 229, 182, 64, 77, 9, 19, 100, 229, 93, 14, 24, 4, 55, 87,
    //     94, 106, 35, 135, 125, 108,
    // ];
    // eth_sign.so
    let code_hash: [u8; 32] = [
        114,136,18,7,241,131,151,251,114,137,71,94,28,208,216,64,104,55,4,5,126,140,166,6,43,114,139,209,174,122,155,68
                                    ];

    let type_no = 0i32;
    // let message: Vec<_> = hex::decode("1c4dce867a1cfa5398f488672506df374bded46a7b0bb922e35d0af40ad4903f").unwrap();
    // let lock_bytes: Vec<_> = hex::decode("b6f5ec0f27014a050594a70e8abe0db2bb4873d5c45da35b5a00629b23aa0eb33c1e3246e63a50b9572e2ec5739665ee9b46b5558e35bab47754fe0d5ae6428401").unwrap();
    // let lock_args: Vec<_> = hex::decode("c9f53b1d85356b60453f867610888d89a0b667ad").unwrap();

    let s: &str = "from did: 0x24d166cd6c8b826c779040b49d5b6708d649b236558e8744339dfee6afe11999";
    let message: Vec<_> = s.as_bytes().to_vec();
    let lock_bytes: Vec<_> = hex::decode("4c491c82a82840bf99bc7b3b45b147ec6e9464b8dedc9e90e6d912c6a3577b9b29d573e16c7ea304e09dfee61f639508c5ca4f17a93e46184c434838945c310b01").unwrap();
    let lock_args: Vec<_> = hex::decode("3a6cab3323833f53754db4202f5741756c436ede").unwrap();

    let mut context = unsafe { CKBDLContext::<[u8; 128 * 1024]>::new() };
    let lib = SignLib::load(&mut context, &code_hash);
    let l = s.len();
    let ret = lib.validate_str(type_no, message, l, lock_bytes, lock_args);
    if let Err(error_code) = ret {
        debug!("Validation failed, error_code: {}", error_code);
    } else {
        debug!("Validation succeed.");
    }

    Ok(())
}

pub fn main() -> Result<(), Error> {
    debug!("====== Running playground ======");

    // let code_hash = [
    //     175, 146, 8, 214, 187, 74, 62, 167, 31, 178, 214, 207, 187, 168, 151, 8, 161, 157, 84, 85, 252, 192, 199, 248,
    //     109, 11, 195, 253, 232, 200, 115, 114,
    // ];
    // // Create a DL context with 64K buffer.
    // let mut context = unsafe { CKBDLContext::<[u8; 32 * 1024]>::new() };
    // // Load library
    // let lib = context.load(&code_hash).expect("load shared lib");

    // // get symbols
    dynamic_loading()?; 
    Ok(())
}
