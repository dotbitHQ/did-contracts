use super::util::sandbox::Sandbox;
use std::fs::File;
use std::io::Read;

#[test]
fn test_sandbox() {
    let mut out_point_map_json = String::new();
    File::open(format!("./out_point_map.json"))
        .expect("Expect out_point_map.json exists.")
        .read_to_string(&mut out_point_map_json)
        .expect("Expect out_point_map.json file readable.");

    let mut tx_json = String::new();
    File::open(format!("./tx.json"))
        .expect("Expect tx.json exists.")
        .read_to_string(&mut tx_json)
        .expect("Expect tx.json file readable.");

    let mut sandbox = Sandbox::new("http://127.0.0.1:8214", &out_point_map_json, &tx_json).unwrap();

    let cycles = sandbox.run().unwrap();

    println!("Run transaction costs: {} cycles", cycles);
}
