#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use ckb_testtool::ckb_types::bytes;

#[macro_use]
mod util;
mod ckb_types_relay;

#[cfg(test)]
mod account_cell_type;
#[cfg(test)]
mod account_sale_cell_type;
#[cfg(test)]
mod apply_register_cell_type;
#[cfg(test)]
mod balance_cell_type;
#[cfg(test)]
mod config_cell_type;
#[cfg(test)]
mod income_cell_type;
#[cfg(test)]
mod offer_cell_type;
#[cfg(test)]
mod playground;
#[cfg(test)]
mod pre_account_cell_type;
#[cfg(test)]
mod proposal_cell_type;
#[cfg(test)]
mod reverse_record_cell_type;
#[cfg(test)]
mod sub_account_cell_type;
#[cfg(test)]
mod witness_parser;

#[cfg(test)]
mod gen_type_id_table;

const BINARY_VERSION: &str = "BINARY_VERSION";

pub enum BinaryVersion {
    Debug,
    Release,
}

impl FromStr for BinaryVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(BinaryVersion::Debug),
            "release" => Ok(BinaryVersion::Release),
            _ => Err("Environment variable BINARY_VERSION only support \"debug\" and \"release\"."),
        }
    }
}

pub struct Loader(PathBuf);

impl Default for Loader {
    fn default() -> Self {
        let test_env = match env::var(BINARY_VERSION) {
            Ok(val) => val.parse().expect("Binary version should be one of debug and release."),
            Err(_) => BinaryVersion::Debug,
        };
        Self::with_test_env(test_env)
    }
}

impl Loader {
    fn with_test_env(env: BinaryVersion) -> Self {
        let load_prefix = match env {
            BinaryVersion::Debug => "debug",
            BinaryVersion::Release => "release",
        };
        let dir = env::current_dir().unwrap();
        let mut base_path = PathBuf::new();
        base_path.push(dir);
        base_path.push("..");
        base_path.push("build");
        base_path.push(load_prefix);
        Loader(base_path)
    }

    fn with_deployed_scripts() -> Self {
        let dir = env::current_dir().unwrap();
        let mut base_path = PathBuf::new();
        base_path.push(dir);
        base_path.push("..");
        base_path.push("deployed-scripts");
        Loader(base_path)
    }

    pub fn load_binary(&self, name: &str) -> bytes::Bytes {
        let mut path = self.0.clone();
        path.push(name);
        fs::read(path.clone())
            .expect(format!("Can not load binary of {} from path {}.", name, path.to_str().unwrap()).as_str())
            .into()
    }
}
