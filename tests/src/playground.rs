use crate::util::{constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};
use ckb_testtool::ckb_hash::blake2b_256;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("playground", false);
    // template.push_shared_lib_cell("ckb_smt.so", false);
    template.push_shared_lib_cell("eth_sign.so", false);
    template.push_shared_lib_cell("secp256k1_data", true);
    template
}

#[test]
fn test_playground() {
    let mut template = init("playground");

    push_input_playground_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn test_smt_verify() {
    use sparse_merkle_tree::{
        ckb_smt::{SMTBuilder, SMT},
        H256
    };

    let root_hash = H256::from([
        26, 255, 57, 237, 95, 177, 194, 59, 79, 143, 56, 114, 219, 17, 57, 191, 214, 49, 183, 60, 14, 45,       44, 101, 167, 67, 194, 27, 98, 68, 199, 46  
    ]);

    let key = H256::from([
        35, 39, 151, 104, 126, 195, 63, 86, 168, 88, 152, 230, 236, 75, 146, 78, 65, 193, 18, 38, 122, 141, 162, 51, 42, 58, 32, 61, 19, 233, 11, 88 
    ]);
    let val = H256::from([
        35, 170, 60, 60, 144, 195, 7, 48, 102, 130, 3, 182, 179, 244, 135, 57, 12, 225, 6, 169, 246, 125, 32, 198, 108, 107, 202, 217, 145, 67, 170, 177
    ]);

    let key_not_exist = H256::from([
        4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);
    let val_not_exist = H256::from([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);

    let proof = [
        76, 79, 253, 81, 253, 52, 50, 132, 225, 29, 228, 94, 128, 41, 152, 40, 150, 132, 145, 101, 248, 78,     17, 45, 14, 32, 112, 65, 179, 137, 143, 114, 133, 174, 62, 110, 245, 91, 188, 110, 15, 128, 104,        148, 171, 83, 108, 156, 199, 122, 145, 190, 146, 84, 150, 206, 158, 161, 39, 206, 84, 182, 81, 148,     9, 34, 103, 13, 27, 79, 2
    ];

    let builder = SMTBuilder::new();
    // let builder = builder.insert(&key_not_exist, &val_not_exist).unwrap();
    let builder = builder.insert(&key, &val).unwrap();

    let smt = builder.build().unwrap();
    let ret = smt.verify(&root_hash, &proof);
    if let Err(e) = ret {
        println!("Verification failed: {}", e);
    } else {
        println!("Verification passed.");
    }

}