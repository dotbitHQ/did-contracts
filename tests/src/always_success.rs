use crate::constants::MAX_CYCLES;
use crate::util::{deploy_contract, mock_cell, mock_input, mock_output, mock_script};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, prelude::*};

#[test]
fn should_always_success() {
    let mut context = Context::default();
    let out_point = deploy_contract(&mut context, "always_success");

    // deploy contract
    let (lock_script, cell_dep) = mock_script(&mut context, out_point, Bytes::default());

    // prepare transaction structure
    let out_point = mock_cell(&mut context, 1000, lock_script.clone(), None, None);
    let input = mock_input(out_point, None);
    let outputs = vec![
        mock_output(500, lock_script.clone(), None),
        mock_output(500, lock_script.clone(), None),
    ];
    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    let tx = TransactionBuilder::default()
        .cell_dep(cell_dep)
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("always_success: {} cycles", cycles);
}
