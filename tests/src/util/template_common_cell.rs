use super::template_generator::TemplateGenerator;
use serde_json::json;

pub fn push_input_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
}
