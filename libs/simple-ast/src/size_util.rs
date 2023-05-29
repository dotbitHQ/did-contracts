use log::debug;

use crate::types::*;

const MOL_HEADER_LENGTH_SIZE: usize = 4;
const MOL_HEADER_OFFSET_SIZE: usize = 4;

pub fn calc_rules_size(rules: &[SubAccountRule]) -> usize {
    let mut size = MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * rules.len();
    for (i, rule) in rules.iter().enumerate() {
        size += calc_rule_size(format!("rules[{}]", i), rule);
    }

    debug!("L{} total size: {}", line!(), size);

    size
}

pub fn calc_rule_size(key: String, rule: &SubAccountRule) -> usize {
    let size = MOL_HEADER_LENGTH_SIZE
        + MOL_HEADER_OFFSET_SIZE + 4 // these are bytes for index field
        + MOL_HEADER_OFFSET_SIZE + calc_string_size(key.clone() + ".name", &rule.name) // these are bytes for name field
        + MOL_HEADER_OFFSET_SIZE + calc_string_size(key.clone() + ".note", &rule.note) // these are bytes for note field
        + MOL_HEADER_OFFSET_SIZE + 8 // these are bytes for price field
        + MOL_HEADER_OFFSET_SIZE + 1 // these are bytes for status field
        + MOL_HEADER_OFFSET_SIZE + calc_expression_size(key.clone() + ".ast", &rule.ast); // these are bytes for ast field

    debug!("L{} {}: {}", line!(), key, size);

    size
}

pub fn calc_expression_size(key: String, expression: &Expression) -> usize {
    let mut size = MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * 2 // these are bytes for ASTExpression header
        + 1  // 1 byte for expression_type
        + MOL_HEADER_LENGTH_SIZE; // the header of Bytes
    match expression {
        Expression::Operator(operator_expr) => {
            size += calc_operator_size(key.clone(), operator_expr);
        }
        Expression::Function(function_expr) => {
            size += calc_function_size(key.clone(), function_expr);
        }
        Expression::Variable(variable_expr) => {
            size += calc_variable_size(key.clone(), variable_expr);
        }
        Expression::Value(value_expr) => {
            size += calc_value_size(key.clone(), value_expr);
        }
    }

    debug!("L{} {}: {}", line!(), key, size);

    size
}

pub fn calc_operator_size(key: String, operator_expr: &OperatorExpression) -> usize {
    let mut size = MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * 2 // these are bytes for ASTOperator header
    + MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * operator_expr.expressions.len() // the header of ASTExpressions
    + 1; // 1 byte for symbol
    for (i, expr) in operator_expr.expressions.iter().enumerate() {
        size += calc_expression_size(format!("{}.expressions[{}]", key, i), expr);
    }

    debug!("L{} {}: {}", line!(), key, size);

    size
}

pub fn calc_function_size(key: String, function_expr: &FunctionExpression) -> usize {
    let mut size = MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * 2 // these are bytes for ASTFunction header
        + MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * function_expr.arguments.len() // the header of ASTExpressions
        + 1; // 1 byte for name
    for (i, expr) in function_expr.arguments.iter().enumerate() {
        size += calc_expression_size(format!("{}.arguments[{}]", key, i), expr);
    }

    debug!("L{} {}: {}", line!(), key, size);

    size
}

pub fn calc_variable_size(key: String, _variable_expr: &VariableExpression) -> usize {
    let size = MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE + 1;

    debug!("L{} {}: {}", line!(), key, size);

    size
}

fn calc_value_size(key: String, value_expr: &ValueExpression) -> usize {
    let mut size = MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * 2 // these are bytes for ASTValue header
        + 1; // 1 byte for value_type
    match value_expr.value {
        Value::Bool(_) => {
            size += MOL_HEADER_LENGTH_SIZE + 1;
        }
        Value::Uint8(_) => {
            size += MOL_HEADER_LENGTH_SIZE + 1;
        }
        Value::Uint32(_) => {
            size += MOL_HEADER_LENGTH_SIZE + 4;
        }
        Value::Uint64(_) => {
            size += MOL_HEADER_LENGTH_SIZE + 8;
        }
        Value::String(ref s) => {
            size += calc_string_size(key.clone(), s);
        }
        Value::StringVec(ref str_vec) => {
            size += MOL_HEADER_LENGTH_SIZE + // the header of Bytes
                MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * str_vec.len(); // the header of BytesVec
            for s in str_vec {
                size += calc_string_size(key.clone(), s);
            }
        }
        Value::Binary(ref b) => {
            size += calc_binary_size(key.clone(), b);
        }
        Value::BinaryVec(ref bin_vec) => {
            size += MOL_HEADER_LENGTH_SIZE + // the header of Bytes
                MOL_HEADER_LENGTH_SIZE + MOL_HEADER_OFFSET_SIZE * bin_vec.len(); // the header of BytesVec
            for b in bin_vec {
                size += calc_binary_size(key.clone(), b);
            }
        }
        Value::CharsetType(_) => {
            size += MOL_HEADER_LENGTH_SIZE + 4;
        }
    }

    debug!("L{} {}: {}", line!(), key, size);

    size
}

pub fn calc_string_size(key: String, input: &str) -> usize {
    calc_binary_size(key, input.as_bytes())
}

pub fn calc_binary_size(key: String, input: &[u8]) -> usize {
    let size = MOL_HEADER_LENGTH_SIZE + input.len();

    debug!("L{} {}: {}", line!(), key, size);

    size
}

#[cfg(test)]
mod test {
    use das_types_std::constants::CharSetType;
    use das_types_std::packed;

    use super::super::util;
    use super::*;

    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn test_calc_rules_size() {
        let rules = vec![
            SubAccountRule {
                index: 0,
                name: String::new(),
                note: String::new(),
                price: 0,
                status: SubAccountRuleStatus::On,
                ast: Expression::Operator(OperatorExpression {
                    symbol: SymbolType::And,
                    expressions: vec![],
                }),
            },
            SubAccountRule {
                index: 0,
                name: String::new(),
                note: String::new(),
                price: 0,
                status: SubAccountRuleStatus::On,
                ast: Expression::Operator(OperatorExpression {
                    symbol: SymbolType::And,
                    expressions: vec![],
                }),
            },
        ];

        let mol = util::sub_account_rules_to_mol_entity(rules.clone()).unwrap();
        let size = calc_rules_size(&rules);
        // println!("mol = {:?}", hex::encode(mol.as_slice()));
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_rule_size() {
        let rule = SubAccountRule {
            index: 0,
            name: String::new(),
            note: String::new(),
            price: 0,
            status: SubAccountRuleStatus::On,
            ast: Expression::Operator(OperatorExpression {
                symbol: SymbolType::And,
                expressions: vec![],
            }),
        };

        let mol: packed::SubAccountRule = rule.clone().into();
        let size = calc_rule_size(String::new(), &rule);
        // println!("mol = {:?}", hex::encode(mol.as_slice()));
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_expression_size() {
        let expression: Expression = Expression::Operator(OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            })],
        });

        let mol: packed::ASTExpression = expression.clone().into();
        let size = calc_expression_size(String::new(), &expression);
        // println!("mol = {:?}", hex::encode(mol.as_slice()));
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_operator_size() {
        let expression = OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![],
        };

        let mol: packed::ASTOperator = expression.clone().into();
        let size = calc_operator_size(String::new(), &expression);
        // println!("mol = {:?}", hex::encode(mol.as_slice()));
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_function_size() {
        let expression = FunctionExpression {
            name: FnName::InList,
            arguments: vec![],
        };
        let mol: packed::ASTFunction = expression.clone().into();
        let size = calc_function_size(String::new(), &expression);
        // println!("mol = {:?}", hex::encode(mol.as_slice()));
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_variable_size() {
        let expression = VariableExpression { name: VarName::Account };
        let mol: packed::ASTVariable = expression.clone().into();
        let size = calc_variable_size(String::new(), &expression);
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_value_size() {
        let expressions = vec![
            ValueExpression {
                value_type: ValueType::Binary,
                value: Value::Bool(true),
            },
            ValueExpression {
                value_type: ValueType::Uint8,
                value: Value::Uint8(1),
            },
            ValueExpression {
                value_type: ValueType::Uint32,
                value: Value::Uint32(1),
            },
            ValueExpression {
                value_type: ValueType::Uint64,
                value: Value::Uint64(1),
            },
            ValueExpression {
                value_type: ValueType::String,
                value: Value::String(String::from("test")),
            },
            ValueExpression {
                value_type: ValueType::StringVec,
                value: Value::StringVec(vec![String::from("test"), String::from("test"), String::from("test")]),
            },
            ValueExpression {
                value_type: ValueType::Binary,
                value: Value::Binary(vec![0; 10]),
            },
            ValueExpression {
                value_type: ValueType::BinaryVec,
                value: Value::BinaryVec(vec![vec![0; 10], vec![0; 10], vec![0; 10]]),
            },
            ValueExpression {
                value_type: ValueType::CharsetType,
                value: Value::CharsetType(CharSetType::Emoji),
            },
        ];

        for expr in expressions.into_iter() {
            let mol: packed::ASTValue = expr.clone().into();
            let size = calc_value_size(String::new(), &expr);
            // println!("type = {:?}", expr.value_type);
            // println!("mol = {:?}", hex::encode(mol.as_slice()));
            assert_eq!(size, mol.total_size());
        }
    }

    #[test]
    fn test_calc_binary_size() {
        let mol = packed::Bytes::from("test".as_bytes());
        let size = calc_binary_size(String::new(), "test".as_bytes());
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn test_calc_string_size() {
        let mol = packed::Bytes::from("test".as_bytes());
        let size = calc_string_size(String::new(), "test");
        assert_eq!(size, mol.total_size());
    }

    #[test]
    fn calc_size() {}
}
