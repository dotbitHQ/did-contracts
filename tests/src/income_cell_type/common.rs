use crate::util::{constants::*, template_generator::*};
use das_types::constants::DataType;

macro_rules! push_income_cell {
    ( $template:expr, $records_param:expr, $index:expr, $source:expr ) => {{
        let (cell_data, entity) =
            $template.gen_income_cell_data("0x0000000000000000000000000000000000000000", $records_param.clone());
        $template.push_income_cell(
            cell_data,
            Some((1, $index, entity)),
            $records_param
                .iter()
                .map(|item| item.capacity)
                .reduce(|a, b| a + b)
                .unwrap(),
            $source,
        );
    }};
}

macro_rules! push_income_cell_with_das_lock {
    ( $template:expr, $records_param:expr, $index:expr, $source:expr ) => {{
        let (cell_data, entity) = $template
            .gen_income_cell_data_with_das_lock("0x0000000000000000000000000000000000000000", $records_param.clone());
        $template.push_income_cell(
            cell_data,
            Some((1, $index, entity)),
            $records_param
                .iter()
                .map(|item| item.capacity)
                .reduce(|a, b| a + b)
                .unwrap(),
            $source,
        );
    }};
}

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("income-cell-type", false);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);

    template
}
