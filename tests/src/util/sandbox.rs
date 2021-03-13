use super::super::util::constants::MAX_CYCLES;
use super::super::Loader;
use ckb_testtool::context::Context;
use ckb_tool::{
    ckb_jsonrpc_types as rpc_types,
    ckb_types::{bytes, core::TransactionBuilder, packed::*, prelude::*},
    rpc_client::RpcClient,
};
use std::collections::HashMap;
use std::error::Error;

pub struct Sandbox {
    context: Context,
    rpc_client: RpcClient,
    tx: rpc_types::Transaction,
    out_point_table: HashMap<OutPoint, String>,
}

impl Sandbox {
    pub fn new<'a>(
        out_point_table: HashMap<OutPoint, String>,
        rpc_url: &'a str,
        tx_json: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let rpc_client = RpcClient::new(rpc_url);
        let tx = serde_json::from_str::<rpc_types::Transaction>(tx_json)?;

        // Check if rpc works.
        rpc_client.get_blockchain_info();

        println!("\n====== Sandbox initialization ======");

        Ok(Sandbox {
            context: Context::default(),
            rpc_client,
            tx,
            out_point_table,
        })
    }

    pub fn run(&mut self) -> Result<u64, Box<dyn Error>> {
        let cell_deps = self.parse_cell_deps()?;
        let inputs = self.parse_inputs()?;
        let outputs = self.parse_outputs()?;
        let outputs_data = self.parse_outputs_data()?;
        let witnesses = self.parse_witnesses()?;

        let tx = TransactionBuilder::default()
            .cell_deps(cell_deps)
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data)
            .set_witnesses(witnesses)
            .build();

        // println!(
        //     "{}",
        //     serde_json::to_string(&rpc_types::TransactionView::from(tx.clone()))?.as_str()
        // );

        println!("====== Run transaction in sandbox ======");

        let cycles = self
            .context
            .verify_tx(&tx, MAX_CYCLES)
            .expect("pass verification");

        Ok(cycles)
    }

    fn mock_cell(
        &mut self,
        _field_name: &str,
        _field_index: usize,
        out_point_rpc: rpc_types::OutPoint,
    ) -> Result<(), Box<dyn Error>> {
        let ret = self
            .rpc_client
            .get_transaction(out_point_rpc.tx_hash.clone())
            .ok_or("Load transaction from node failed.")?;

        // Create the Cell of the OutPoint from online transaction in context.
        let index = out_point_rpc.index.value();
        let cell = CellOutput::from(
            ret.transaction
                .inner
                .outputs
                .get(index as usize)
                .unwrap()
                .to_owned(),
        );
        let cell_data = Bytes::from(
            ret.transaction
                .inner
                .outputs_data
                .get(index as usize)
                .unwrap()
                .to_owned(),
        );

        let out_point = OutPoint::from(out_point_rpc.clone());

        // println!(
        //     "{}[{}] out_point = {}",
        //     _field_name,
        //     _field_index,
        //     serde_json::to_string(&rpc_types::OutPoint::from(out_point.clone()))?.as_str()
        // );
        // println!(
        //     "{}[{}] cell = {}",
        //     _field_name,
        //     _field_index,
        //     serde_json::to_string(&rpc_types::CellOutput::from(cell.clone()))?.as_str()
        // );

        let ret = self.out_point_table.get(&out_point);
        // If out_point is refer to a script and it is not in the cell_deps field, then load the script code into cell.
        if ret.is_some() {
            let filename = ret.unwrap();
            let contract_bin: bytes::Bytes;
            if ["secp256k1_blake160_sighash_all", "secp256k1_data"].contains(&filename.as_str()) {
                println!(
                    "  {}[{}] mock from builtin contract: {}",
                    _field_name, _field_index, filename
                );
                contract_bin = Loader::with_deployed_scripts().load_binary(filename);
            } else {
                println!(
                    "  {}[{}] mock from developed contract: {}",
                    _field_name, _field_index, filename
                );
                contract_bin = Loader::default().load_binary(filename);
            }

            self.context
                .create_cell_with_out_point(out_point, cell, contract_bin);
        } else {
            println!(
                "  {}[{}] mock from online outputs_data.",
                _field_name, _field_index
            );

            self.context
                .create_cell_with_out_point(out_point, cell, cell_data.unpack())
        }

        Ok(())
    }

    fn parse_cell_deps(&mut self) -> Result<Vec<CellDep>, Box<dyn Error>> {
        let mut cell_deps = Vec::new();

        let cell_deps_rpc = self.tx.cell_deps.clone();
        for (i, item) in cell_deps_rpc.iter().enumerate() {
            if item.dep_type == rpc_types::DepType::Code {
                self.mock_cell("cell_deps", i, item.out_point.clone())?;
            } else {
                // Mock the cell which dep_type = DepType::Group .
                self.mock_cell("cell_deps", i, item.out_point.clone())?;

                // Mock the cells which included in previous cell's data .
                let index = item.out_point.index.value();
                let ret = self
                    .rpc_client
                    .get_transaction(item.out_point.tx_hash.clone())
                    .ok_or("Load transaction from node failed.")?;

                let raw_out_points = ret
                    .transaction
                    .inner
                    .outputs_data
                    .get(index as usize)
                    .unwrap()
                    .to_owned();
                let out_points = OutPointVec::from_slice(raw_out_points.as_bytes())?;

                println!("  ====== cell_deps[{}] group expanding ======", i);

                for out_point in out_points.into_iter() {
                    self.mock_cell("cell_deps", i, rpc_types::OutPoint::from(out_point.clone()))?;
                }

                println!("  ====== cell_deps[{}] group expanded ======", i);
            }

            let cell_dep = CellDep::from(item.to_owned());
            cell_deps.push(cell_dep);
        }

        Ok(cell_deps)
    }

    fn parse_inputs(&mut self) -> Result<Vec<CellInput>, Box<dyn Error>> {
        let mut inputs = Vec::new();

        let inputs_rpc = self.tx.inputs.clone();
        for (i, item) in inputs_rpc.iter().enumerate() {
            self.mock_cell("inputs", i, item.previous_output.clone())?;
            let input = CellInput::from(item.to_owned());
            inputs.push(input);
        }

        Ok(inputs)
    }

    fn parse_outputs(&mut self) -> Result<Vec<CellOutput>, Box<dyn Error>> {
        let mut outputs = Vec::new();

        let outputs_rpc = self.tx.outputs.clone();
        for item in outputs_rpc.iter() {
            let output = CellOutput::from(item.to_owned());
            outputs.push(output);
        }

        Ok(outputs)
    }

    fn parse_outputs_data(&mut self) -> Result<Vec<Bytes>, Box<dyn Error>> {
        let mut outputs_data = Vec::new();

        let outputs_data_rpc = self.tx.outputs_data.clone();
        for item in outputs_data_rpc.iter() {
            let data = Bytes::from(item.to_owned());
            outputs_data.push(data);
        }

        Ok(outputs_data)
    }

    fn parse_witnesses(&mut self) -> Result<Vec<Bytes>, Box<dyn Error>> {
        let mut witnesses = Vec::new();

        let witnesses_rpc = self.tx.witnesses.clone();
        for item in witnesses_rpc.iter() {
            let data = Bytes::from(item.to_owned());
            witnesses.push(data);
        }

        Ok(witnesses)
    }
}
