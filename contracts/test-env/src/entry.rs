use alloc::boxed::Box;
use alloc::string::ToString;
use core::result::Result;

use ckb_std::debug;
use das_core::error::*;
use das_core::{code_to_error, warn};
use das_types::constants::*;
use witness_parser::WitnessesParserV1;

// use simple_ast::types as ast_types;
use crate::{config_tests, uint_tests, witness_parser_tests};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running test-env ======");

    let parser = WitnessesParserV1::get_instance();
    parser.init().map_err(|_err| {
        debug!("_err: {:?}", _err);
        code_to_error!(ErrorCode::WitnessDataDecodingError)
    })?;

    if parser.action != Action::UnitTest {
        warn!("Action is undefined: {:?}", parser.action.to_string());
        return Err(code_to_error!(ErrorCode::ActionNotSupported));
    }
    let test_name = match &parser.action_params {
        ActionParams::TestName(name) => name,
        _ => {
            warn!("ActionParams is invalid");
            return Err(code_to_error!(ErrorCode::HardCodedError));
        }
    };

    debug!("Route to {:?} test ...", test_name);

    match test_name.as_str() {
        "test_uint_basic_interface" => uint_tests::test_basic_interface()?,
        "test_uint_safty" => uint_tests::test_safty()?,
        "perf_uint_price_formula" => uint_tests::perf_price_formula()?,
        "test_config_account_loading" => config_tests::test_config_account_loading()?,
        "test_config_records_key_namespace_loading" => config_tests::test_config_records_key_namespace_loading()?,
        "test_witness_parser_get_entity_by_cell_meta" => {
            witness_parser_tests::test_witness_parser_get_entity_by_cell_meta()?
        }
        "test_parse_sub_account_witness_empty" => {
            witness_parser_tests::sub_account::test_parse_sub_account_witness_empty()?
        }
        "test_parse_sub_account_witness_create_only" => {
            witness_parser_tests::sub_account::test_parse_sub_account_witness_create_only()?
        }
        "test_parse_sub_account_witness_edit_only" => {
            witness_parser_tests::sub_account::test_parse_sub_account_witness_edit_only()?
        }
        "test_parse_sub_account_witness_mixed" => {
            witness_parser_tests::sub_account::test_parse_sub_account_witness_mixed()?
        }
        "test_parse_sub_account_rules_witness_empty" => {
            witness_parser_tests::sub_account::test_parse_sub_account_rules_witness_empty()?
        }
        "test_parse_sub_account_rules_witness_simple" => {
            witness_parser_tests::sub_account::test_parse_sub_account_rules_witness_simple()?
        }
        "test_parse_reverse_record_witness_empty" => {
            witness_parser_tests::reverse_record::test_parse_reverse_record_witness_empty()?
        }
        "test_parse_reverse_record_witness_update_only" => {
            witness_parser_tests::reverse_record::test_parse_reverse_record_witness_update_only()?
        }
        "test_parse_reverse_record_witness_remove_only" => {
            witness_parser_tests::reverse_record::test_parse_reverse_record_witness_remove_only()?
        }
        "test_parse_reverse_record_witness_mixed" => {
            witness_parser_tests::reverse_record::test_parse_reverse_record_witness_mixed()?
        }
        _ => {
            warn!("Test not found: {:?}", test_name);
            return Err(code_to_error!(ErrorCode::HardCodedError));
        }
    }

    Ok(())
}
