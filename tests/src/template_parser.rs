// Suppress warning here is because it is mistakenly treat the code as dead code when running unit tests.
#![allow(dead_code)]

use crate::util::{
    deploy_builtin_contract, deploy_contract, hex_to_byte32, hex_to_bytes, hex_to_u64, mock_cell,
    mock_input, mock_script,
};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    core::TransactionBuilder,
    core::TransactionView,
    packed::*,
    prelude::{Builder, Entity, Pack},
};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

lazy_static! {
    static ref VARIABLE_REG: Regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
}

#[derive(Debug)]
pub struct Contract {
    script: Script,
    cell_dep: CellDep,
}

pub struct TemplateParser<'a> {
    context: &'a mut Context,
    data: Value,
    contracts: HashMap<String, Contract>,
    inputs: Vec<CellInput>,
    outputs: Vec<CellOutput>,
    outputs_data: Vec<Bytes>,
}

impl std::fmt::Debug for TemplateParser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateParser")
            .field("contracts", &self.contracts)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .field("outputs_data", &self.outputs_data)
            .finish()
    }
}

impl<'a> TemplateParser<'a> {
    pub fn new(context: &'a mut Context, raw_json: &str) -> Result<Self, Box<dyn Error>> {
        let data: Value = serde_json::from_str(raw_json)?;

        Ok(TemplateParser {
            context,
            data,
            contracts: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            outputs_data: Vec::new(),
        })
    }

    pub fn parse(&mut self) -> () {
        if let Err(e) = self.try_parse() {
            panic!(format!("{}", e.to_string()));
        }
    }

    pub fn try_parse(&mut self) -> Result<(), Box<dyn Error>> {
        let to_owned = |v: &Vec<Value>| -> Vec<Value> { v.to_owned() };

        if let Some(cell_deps) = self.data["cell_deps"].as_array().map(to_owned) {
            self.parse_cell_deps(cell_deps)?
        }
        if let Some(inputs) = self.data["inputs"].as_array().map(to_owned) {
            self.parse_inputs(inputs)?
        }
        if let Some(outputs) = self.data["outputs"].as_array().map(to_owned) {
            self.parse_outputs(outputs)?
        }

        Ok(())
    }

    pub fn set_outputs_data(&mut self, i: usize, data: Bytes) {
        self.outputs_data[i] = data;

        eprintln!("Set self.outputs_data = {:#?}", self.outputs_data);
    }

    pub fn build_tx(&mut self) -> TransactionView {
        let cell_deps = self
            .contracts
            .iter()
            .map(|(_, contract)| contract.cell_dep.clone())
            .collect::<Vec<_>>();

        TransactionBuilder::default()
            .cell_deps(cell_deps)
            .inputs(self.inputs.clone())
            .outputs(self.outputs.clone())
            .outputs_data(self.outputs_data.pack())
            .build()
    }

    fn parse_cell_deps(&mut self, cell_deps: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for item in cell_deps {
            let name = item["tmp_file_name"].as_str().unwrap();
            let out_point;

            match item["tmp_type"].as_str() {
                Some("contract") => out_point = deploy_contract(self.context, name),
                Some("deployed_contract") => {
                    out_point = deploy_builtin_contract(self.context, name)
                }
                _ => {
                    return Err("Unsupported cell_deps type.".into());
                }
            }

            let (script, cell_dep) = mock_script(self.context, out_point, Bytes::default());

            self.contracts
                .insert(name.to_string(), Contract { script, cell_dep });
        }

        eprintln!("Parse self.contracts = {:#?}", self.contracts);
        Ok(())
    }

    fn parse_inputs(&mut self, inputs: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for item in inputs {
            match item["previous_output"]["tmp_type"].as_str() {
                Some("full") => {
                    // parse inputs[].previous_output as a mock cell
                    let (capacity, lock_script, type_script, data) = self
                        .parse_cell(item["previous_output"].clone())
                        .map_err(|err| {
                            format!(
                                "Field `inputs[].previous_output` parse failed: {}",
                                err.to_string()
                            )
                        })?;
                    let out_point =
                        mock_cell(self.context, capacity, lock_script, type_script, data);

                    // parse input.since
                    let since;
                    if item["since"].is_number() {
                        since = item["since"].as_u64();
                    } else {
                        let hex = item["since"].as_str();
                        since = hex.map(|hex| {
                            hex_to_u64(hex)
                                .expect("Field `inputs[].since` is not valid u64 in hex.")
                        });
                    }

                    self.inputs.push(mock_input(out_point, since));
                }
                _ => {
                    return Err("Unsupported inputs type.".into());
                }
            }
        }

        eprintln!("Parse self.inputs = {:#?}", self.inputs);
        Ok(())
    }

    fn parse_outputs(&mut self, outputs: Vec<Value>) -> Result<(), Box<dyn Error>> {
        for item in outputs {
            match item["tmp_type"].as_str() {
                Some("full") => {
                    // parse inputs[].previous_output as a mock cell
                    let (capacity, lock_script, type_script, data) =
                        self.parse_cell(item.clone()).map_err(|err| {
                            format!("Field `outputs[]` parse failed: {}", err.to_string())
                        })?;

                    let cell: CellOutput = CellOutput::new_builder()
                        .capacity(capacity.pack())
                        .lock(lock_script)
                        .type_(ScriptOpt::new_builder().set(type_script).build())
                        .build();

                    self.outputs.push(cell);
                    self.outputs_data.push(data.unwrap_or(Bytes::default()));
                }
                _ => {
                    return Err("Unsupported inputs type.".into());
                }
            }
        }

        eprintln!("Parse self.outputs = {:#?}", self.outputs);
        eprintln!("Parse self.outputs_data = {:#?}", self.outputs_data);
        Ok(())
    }

    fn parse_cell(
        &mut self,
        cell: Value,
    ) -> Result<(u64, Script, Option<Script>, Option<Bytes>), Box<dyn Error>> {
        // parse capacity of cell
        let capacity: u64;
        if cell["capacity"].is_number() {
            capacity = cell["capacity"]
                .as_u64()
                .ok_or("Field `cell.capacity` is required.")?;
        } else {
            let hex = cell["capacity"]
                .as_str()
                .ok_or("Field `cell.capacity` is required.")?;
            capacity = hex_to_u64(hex).expect("Field `cell.capacity` is not valid u64 in hex.");
        }

        // parse lock script and type script of cell
        let lock_script = self
            .parse_script(cell["lock"].clone())
            .map_err(|err| format!("Field `cell.lock` parse failed: {}", err.to_string()))?;
        let type_script = self
            .parse_script(cell["type"].clone())
            .map_err(|err| format!("Field `cell.type` parse failed: {}", err.to_string()))?;

        // parse data of cell
        let data;
        if let Some(hex) = cell["data"].as_str() {
            data = Some(hex_to_bytes(hex).map_err(|err| {
                format!(
                    "Field `inputs[].previous_output.tmp_data` parse failed: {}",
                    err.to_string()
                )
            })?)
        } else {
            data = None;
        }

        Ok((capacity, lock_script.unwrap(), type_script, data))
    }

    fn parse_script(&self, script_val: Value) -> Result<Option<Script>, Box<dyn Error>> {
        if script_val.is_null() {
            return Ok(None);
        }

        let script;
        if let Some(code_hash) = script_val["code_hash"].as_str() {
            // If code_hash is variable like {{xxx}}, then parse script field as deployed contract,
            if let Some(caps) = VARIABLE_REG.captures(code_hash) {
                let script_name = caps.get(1).map(|m| m.as_str()).unwrap();
                script = self
                    .contracts
                    .get(script_name)
                    .map(|item| item.script.clone());

            // else parse script field by field.
            } else {
                let code_hash = script_val["code_hash"]
                    .as_str()
                    .expect("The code_hash field is required.");
                let args = script_val["args"]
                    .as_str()
                    .expect("The args field is required.");
                let hash_type = match script_val["hash_type"].as_str() {
                    Some("type") => ScriptHashType::Type,
                    _ => ScriptHashType::Data,
                };

                script = Some(
                    Script::new_builder()
                        .code_hash(hex_to_byte32(code_hash)?)
                        .hash_type(hash_type.into())
                        .args(hex_to_bytes(args)?.pack())
                        .build(),
                );
            }
        } else {
            return Err("The code_hash field is required.".into());
        }

        Ok(script)
    }
}
