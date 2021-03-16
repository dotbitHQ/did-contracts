use super::super::util::constants::MAX_CYCLES;
use super::super::Loader;
use ckb_testtool::context::Context;
use ckb_tool::{
    ckb_jsonrpc_types as rpc_types,
    ckb_types::{bytes, core::TransactionBuilder, packed::*, prelude::*, H256},
    rpc_client::RpcClient,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellType {
    Normal,
    Contract,
    BuiltInNormal,
    BuiltInContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutPointItem {
    cell_type: CellType,
    out_point: Option<rpc_types::OutPoint>,
    type_id: Option<H256>,
    value: String,
}

pub struct Sandbox {
    context: Context,
    rpc_client: RpcClient,
    tx: rpc_types::Transaction,
    out_point_map: HashMap<OutPoint, OutPointItem>,
    type_id_map: HashMap<H256, OutPointItem>,
}

impl Sandbox {
    pub fn new<'a>(
        rpc_url: &'a str,
        out_point_map_json: &str,
        tx_json: &str,
    ) -> Result<Self, Box<dyn Error>> {
        println!("\n====== Sandbox initialization ======");

        let rpc_client = RpcClient::new(rpc_url);
        let out_point_map_items = serde_json::from_str::<Vec<OutPointItem>>(out_point_map_json)?;
        let tx = serde_json::from_str::<rpc_types::Transaction>(tx_json)?;

        // Check if rpc works.
        rpc_client.get_blockchain_info();

        let mut out_point_map = HashMap::new();
        let mut type_id_map = HashMap::new();
        for item in out_point_map_items {
            if item.out_point.is_some() {
                out_point_map.insert(OutPoint::from(item.out_point.clone().unwrap()), item);
            } else if item.type_id.is_some() {
                type_id_map.insert(item.type_id.clone().unwrap(), item);
            } else {
                return Err(
                    "Invalid OutPointItem, either out_point or type_id should be some.".into(),
                );
            }
        }

        Ok(Sandbox {
            context: Context::default(),
            rpc_client,
            tx,
            out_point_map,
            type_id_map,
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
        let mut ret = self.out_point_map.get(&out_point);
        if ret.is_none() {
            ret = match cell.type_().to_opt() {
                Some(type_script) => {
                    let type_id: H256 = type_script.code_hash().unpack();
                    self.type_id_map.get(&type_id)
                }
                None => None,
            };
        }

        // If out_point is refer to a script and it is not in the cell_deps field, then load the script code into cell.
        if ret.is_some() {
            let out_point_item = ret.unwrap();
            let value = &out_point_item.value;
            let contract_bin: bytes::Bytes;

            match out_point_item.cell_type {
                CellType::BuiltInNormal => {
                    println!(
                        "    [{}] is a built-in special cell: {}",
                        _field_index, value
                    );
                    self.context
                        .create_cell_with_out_point(out_point, cell, cell_data.unpack())
                }
                CellType::BuiltInContract => {
                    println!("    [{}] is built-in contract: {}", _field_index, value);
                    contract_bin = Loader::with_deployed_scripts().load_binary(value);
                    self.context
                        .create_cell_with_out_point(out_point, cell, contract_bin);
                }
                CellType::Normal => {
                    println!("    [{}] is a special cell: {}", _field_index, value);
                    self.context
                        .create_cell_with_out_point(out_point, cell, cell_data.unpack())
                }
                CellType::Contract => {
                    println!("    [{}] is developed contract: {}", _field_index, value);
                    contract_bin = Loader::default().load_binary(value);
                    self.context
                        .create_cell_with_out_point(out_point, cell, contract_bin);
                }
            }
        } else {
            if cell.type_().is_none() {
                println!("    [{}] is a very normal cell.", _field_index);
            } else {
                println!("    [{}] is a unknown cell.", _field_index);
            }

            self.context
                .create_cell_with_out_point(out_point, cell, cell_data.unpack())
        }

        Ok(())
    }

    fn distinguish_cell(&mut self, _field_index: usize, cell_rpc: rpc_types::CellOutput) {
        let cell = CellOutput::from(cell_rpc);
        let ret = match cell.type_().to_opt() {
            Some(type_script) => {
                let type_id: H256 = type_script.code_hash().unpack();
                self.type_id_map.get(&type_id)
            }
            None => None,
        };

        // If out_point is refer to a script and it is not in the cell_deps field, then load the script code into cell.
        if ret.is_some() {
            let out_point_item = ret.unwrap();
            let value = &out_point_item.value;

            match out_point_item.cell_type {
                CellType::BuiltInNormal => {
                    println!(
                        "    [{}] is a built-in special cell: {}",
                        _field_index, value
                    );
                }
                CellType::Normal => {
                    println!("    [{}] is a special cell: {}", _field_index, value);
                }
                _ => println!(
                    "    [{}] is a contract cell, and contract cell should not be here.",
                    _field_index
                ),
            }
        } else {
            if cell.type_().is_none() {
                println!("    [{}] is a very normal cell.", _field_index);
            } else {
                println!("    [{}] is a unknown cell.", _field_index);
            }
        }
    }

    fn parse_cell_deps(&mut self) -> Result<Vec<CellDep>, Box<dyn Error>> {
        println!("  cell_deps:");
        let mut cell_deps = Vec::new();

        let cell_deps_rpc = self.tx.cell_deps.clone();
        for (i, item) in cell_deps_rpc.iter().enumerate() {
            if item.dep_type == rpc_types::DepType::Code {
                self.mock_cell(i, item.out_point.clone())?;
            } else {
                // Mock the cell which dep_type = DepType::Group .
                self.mock_cell(i, item.out_point.clone())?;

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

                println!("    ====== cell_deps[{}] group expanding ======", i);

                let mut j = 0;
                for out_point in out_points.into_iter() {
                    self.mock_cell(j, rpc_types::OutPoint::from(out_point.clone()))?;
                    j += 1;
                }

                println!("    ====== cell_deps[{}] group expanded ======", i);
            }

            let cell_dep = CellDep::from(item.to_owned());
            cell_deps.push(cell_dep);
        }

        Ok(cell_deps)
    }

    fn parse_inputs(&mut self) -> Result<Vec<CellInput>, Box<dyn Error>> {
        println!("  inputs:");
        let mut inputs = Vec::new();

        let inputs_rpc = self.tx.inputs.clone();
        for (i, item) in inputs_rpc.iter().enumerate() {
            self.mock_cell(i, item.previous_output.clone())?;
            let input = CellInput::from(item.to_owned());
            inputs.push(input);
        }

        Ok(inputs)
    }

    fn parse_outputs(&mut self) -> Result<Vec<CellOutput>, Box<dyn Error>> {
        println!("  outputs:");
        let mut outputs = Vec::new();

        let outputs_rpc = self.tx.outputs.clone();
        for (i, item) in outputs_rpc.iter().enumerate() {
            self.distinguish_cell(i, item.clone());
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
