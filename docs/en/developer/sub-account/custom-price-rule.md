# Custom Pricing Configuration Structure

## Pricing Rules

> If not a single pricing rule matches successfully, then the account must be treated as unregistrable.

A sub-account can have only one set of pricing rules. A set can contain multiple rules, which are executed in the order they are listed in the array. The first rule that successfully matches will be used as the pricing for the current account.
The fields for a single pricing configuration are as follows:

- `name`, a UTF-8 string, is the name of the current pricing configuration and cannot be empty;
- `note`, a UTF-8 string, provides explanatory information about the current pricing rule and can be empty;
- `price`, an integer, is the price of the account if the current pricing rule matches successfully, in USD. Consistent with other parts of the contract, 1 USD is stored as `1_000_000`;
- `status`, an integer, indicates the status of the rule, where `0x00` is the closed state and `0x01` is the open state;
- `ast`, the abstract syntax tree used for executing matches at the current price. The final calculation result must be of `bool` type. See the section on abstract syntax trees below for details;

### JSON Storage Structure

```json
[
    {
        "index": 0,
        "name": "...",
        "note": "...",
        "price": 0,
        "status": 0,
        "ast": { ... }
    }
]
```

### Molecule Storage Structure

```
table SubAccountRule {
    index: Uint32,
    name: Bytes,
    note: Bytes,
    price: Uint64, // 必定为 0
    ast: ASTExpression,
    status: Uint8,
}

vector SubAccountRules <SubAccountRule>;
```


## Reserved Account Name Rules

A sub-account can have only one set of reserved account name configurations. A configuration can contain multiple rules, which are executed in the order they are listed in the array. Once a match is successful, the account is treated as a reserved account.
The fields for a single rule configuration are as follows:

- `name`, a UTF-8 string, is the name of the current configuration and cannot be empty;
- `note`, a UTF-8 string, provides explanatory information about the current rule and can be empty;
- `status`, an integer, indicates the status of the rule, where `0x00` is the closed state and `0x01` is the open state;
- `ast`, the abstract syntax tree used for executing matches at the current time. The final calculation result must be of `bool` type. See the section on abstract syntax trees below for details;

### JSON Storage Structure

> Reuse the **Pricing Rules** data structure, but the `price` will always be `0`.
。

```json
[
    {
        "index": 0,
        "name": "...",
        "note": "...",
        "price": 0,
        "status": 0,
        "ast": { ... }
    }
]
```

### Molecule Storage Structure

> Reuse the **Pricing Rules** data structure, but the `price` will always be `0`.
。

```
table SubAccountRule {
    index: Uint32,
    name: Bytes,
    note: Bytes,
    price: Uint64,
    ast: ASTExpression,
    status: Uint8,
}

vector SubAccountRules <SubAccountRule>;
```


## Abstract Syntax Tree

The Abstract Syntax Tree (AST) is a structure used to describe configuration rules, which can be understood as a type of programming language based on configuration files.


### expression

In the design of this syntax tree, everything is an expression, and expressions are divided into the following three categories:

- **operator**, this type of expression first calculates its arguments to obtain the following **value**, then performs calculations based on predefined logical operation rules, and returns `true` or `false`;
- **function**, this type of expression first calculates its arguments to obtain the following **value**, then performs calculations based on custom function logic, and returns `true` or `false`;
- **variable**, this type of expression represents built-in variables, such as account names, account length, etc.;
- **value**, this type of expression represents types such as `int, binary, string`, etc., and must be used in conjunction with **operator** or **function**;

### operator

The predefined operators include the following `symbol`s:

- `and`, indicates a logical AND relationship between expressions. This operator can accept multiple expressions, and the type of each expression must be `bool`;
- `or`, indicates a logical OR relationship between expressions. This operator can accept multiple expressions, and the type of each expression must be `bool`;
- `not`, indicates a logical NOT operation on an expression. This operator can accept only 1 expression, and the type of the expression must be `bool`;
- `>`, indicates a greater-than comparison between two expressions. This operator can accept 2 expressions, and the type of each expression must be `uint`, with the first expression on the left and the second expression on the right for comparison;
- `>=`, indicates a greater-than-or-equal-to comparison between two expressions. This operator can accept 2 expressions, and the type of each expression must be `uint`, with the first expression on the left and the second expression on the right for comparison;
- `<`, indicates a less-than comparison between two expressions. This operator can accept 2 expressions, and the type of each expression must be `uint`, with the first expression on the left and the second expression on the right for comparison;
- `<=`, indicates a less-than-or-equal-to comparison between two expressions. This operator can accept 2 expressions, and the type of each expression must be `uint`, with the first expression on the left and the second expression on the right for comparison;
- `==`, indicates an equality comparison between two expressions. This operator can accept 2 expressions, and the type of each expression must be `uint`, with the first expression on the left and the second expression on the right for comparison;

All operators must return a value of the `bool` type upon completion of their execution.

### function

The predefined functions are as follows:

#### include_chars

Check if the account name contains specific characters; if it does, return true, otherwise return false:

```
fn include_chars(account: string, chars: string[]) -> bool;
```

#### include_words

Check if the account name contains a specific string; if it does, return true, otherwise return false:

```
fn include_words(account: string, words: string[]) -> bool;
```

#### starts_with


Check if the account name starts with certain characters; if it does, return true, otherwise return false:

```
fn starts_with(account: string, words: string[]) -> bool;
```

#### ends_with

Check if an account name ends with certain characters. If it does, return true; otherwise, return false:
```
fn ends_with(account: string, words: string[]) -> bool;
```

#### only_include_charset

Check if the characters in an account name consist only of a specific character set. If they do, return true; otherwise, return false ：

```
fn only_include_charset(account_chars: account_chars, charset: charset_type) -> bool;
```

#### include_charset

Check if the characters in an account name contain a specific character set. If they do, return true; otherwise, return false ：

```
fn include_charset(account_chars: account_chars, charset: charset_type) -> bool;
```

#### in_list

Check if the account name exists in the list. If it does, return true; otherwise, return false ：

```
fn in_list(account: string, account_list: binary[]) -> bool;
```

- account_list" is an array formed by calculating the account ID using the "account"；



### variable

The predefined built-in variables include the following:

- `account`, of type string, represents the UTF-8 string of the account name, including the suffix part. For example, the `account` variable for `xxxxx.bit` is `xxxxx.bit`, and for `xxxxx.yyy.bit`, the `account` variable is `xxxxx.yyy.bit`.
- `account_chars`, of type string[], represents the `AccountChars` data structure of the account name, which contains all the information for each character in the account name. This structure, like elsewhere, does not include the suffix part.
- `account_length`, of type uint32, represents the character length of the account name, **which is also the length of the `AccountChars` data structure**.

### value

Predefined value types include the following:

- bool
- `uint8`: Different `uint` types can be converted to larger types for comparison.
- `uint32`: Different `uint` types can be converted to larger types for comparison.
- `uint64`: Different `uint` types can be converted to larger types for comparison.
- `binary`: Corresponds to types like `Buffer` or `Byte` in other languages. It is stored in JSON as a hexadecimal string with a `0x` prefix.
- `binary[]`: Corresponds to types like `Buffer` or `Byte` in other languages. It is stored in JSON as an array of hexadecimal strings with `0x` prefixes.
- `string`: A type for UTF-8 encoded strings.
- `string[]`: A type for arrays of UTF-8 encoded strings.
- `charset_type`: An enumeration of character set types stored as UTF-8 strings. Available values include:

    - Emoji
    - Digit
    - En
    - ZhHans
    - ZhHant
    - Ja
    - Ko
    - Ru
    - Tr
    - Th
    - Vi

> Because in some languages there is a possibility of overflow when parsing uint64 types in JSON, it is supported to store numbers in JSON in string form. You can also use `_` as a separator to improve readability. For example, `1 000 000 000` can be written as `1_000_000_000` or `10_00000000`. The `_` will be ignored when the number is ultimately parsed, and it has no practical significance.


### JSON Storage Structure

```json
{
    "type": "operator",
    "symbol": "and", // and|or|not|...
    "expressions": [
        {expression},
        {expression},
        ...
    ]
}

{
    "type": "function",
    "name": "in_list", // include_chars|include_words|only_include_charset|in_list
    "arguments": [
        {expression},
        {expression},
        {expression},
        ...
    ]
}

{
    "type": "variable",
    "name": "account" // account|account_chars|account_length
}

{
    "type": "value",
    "value_type": "bool", // bool|unit8|uint32|...
    "value": {value}
}
```

### Molecule Storage Structure

```
// Because the molecule do not support recursive type, we can not use union here.
table ASTExpression {
    // Indicate the real type of expression field:
    // - 0x00 ASTOperator
    // - 0x01 ASTFunction
    // - 0x02 ASTVariable
    // - 0x03 ASTValue
    expression_type: byte,
    expression: Bytes,
}

vector ASTExpressions <ASTExpression>;

table ASTOperator {
    // Indicate the operator type:
    // - 0x00 `not`
    // - 0x01 `and`
    // - 0x02 `or`
    // - 0x03 `>`
    // - 0x04 `>=`
    // - 0x05 `<
    // - 0x06 `<=`
    // - 0x07 `==`
    symbol: byte,
    expressions: ASTExpressions,
}

table ASTFunction {
    // Indicate the function name:
    // - 0x00 `include_chars`
    // - 0x01 `include_words`
    // - 0x02 `only_include_charset`
    // - 0x03 `in_list`
    // - 0x04 `include_charset`
    // - 0x05 `starts_with`
    // - 0x06 `ends_with`
    name: byte,
    arguments: ASTExpressions,
}

table ASTVariable {
    // Indicate the variable name:
    // - 0x00 `account`
    // - 0x01 `account_chars`
    // - 0x02 `account_length`
    name: byte,
}

table ASTValue {
    // Indicate the value type
    // - 0x00 bool
    // - 0x01 uint8
    // - 0x02 uint32
    // - 0x03 uint64
    // - 0x04 binary
    // - 0x05 binary[]
    // - 0x06 string
    // - 0x07 string[]
    // - 0x08 charset_type
    value_type: byte,
    value: Bytes,
}
```

In `ASTValue.value` ，according to different storage types, they correspond to the following molecule types:
：

- bool => `byte`, please note that `byte` here is not written in uppercase, as it is the basic type used in molecule encoding.
- uint8 => `Uint8`
- uint32 => `Uint32`
- uint64 => `Uint64`
- binary => `Bytes`
- binary[] => `BytesVec`
- string => `Bytes`
- string[] => `BytesVec`
- charset_type => `Uint32`


## Actual Structure Example

### Pricing Based on Length

```json
[
    {
        "name": "1-Character Accounts",
        "note": "",
        "price": 100000000, // 100 USD
        "ast": {
            "type": "operator",
            "symbol": "==",
            "expressions": [
                {
                    "type": "variable",
                    "name": "account_length"
                },
                {
                    "type": "value",
                    "value_type": "uint8",
                    "value": 1
                }
            ]
        }
    },
    {
        "name": "2-Character Accounts",
        "note": "",
        "price": 10000000, // 10 USD
        "ast": {
            "type": "operator",
            "symbol": "==",
            "expressions": [
                {
                    "type": "variable",
                    "name": "account_length"
                },
                {
                    "type": "value",
                    "value_type": "uint",
                    "value": 2
                }
            ]
        }
    }
    ...
    {
        "name": "Accounts with 8 Characters or More",
        "note": "",
        "price": 100000, // 0.1 USD
        "ast": {
            "type": "operator",
            "symbol": ">=",
            "expressions": [
                {
                    "type": "variable",
                    "name": "account_length"
                },
                {
                    "type": "value",
                    "value_type": "uint",
                    "value": 8
                }
            ]
        }
    }
]
```

### Pricing based on Length and Character Set

```json
[
    {
        "name": "1-Digit Accounts",
        "note": "",
        "price": 100000000, // 100 USD
        "ast": {
            "type": "operator",
            "symbol": "and",
            "expressions": [
                {
                    "type": "operator",
                    "symbol": "==",
                    "expressions": [
                        {
                            "type": "variable",
                            "name": "account_length"
                        },
                        {
                            "type": "value",
                            "value_type": "uint",
                            "value": 1
                        }
                    ]
                },
                {
                    "type": "function",
                    "name": "only_include_charset",
                    "arguments": [
                        {
                            "type": "variable",
                            "name": "account_charts"
                        },
                        {
                            "type": "value",
                            "value_type": "charset_type",
                            "value": "Digit"
                        }
                    ]
                }
            ]
        }
    },
]
```

### Pricing Based on the Presence of Specific Characters

```json
[
    {
        "name": "Special Character Account",
        "note": "",
        "price": 100000000, // 100 USD
        "ast": {
            "type": "function",
            "name": "include_chars",
            "arguments": [
                {
                    "type": "variable",
                    "name": "account"
                },
                {
                    "type": "value",
                    "value_type": "string[]",
                    "value": [
                        "⚠️",
                        "❌",
                        "✅"
                    ]
                }
            ]
        }
    },
]
```

### Pricing According to a Whitelist

```json
[
    {
        "name": "Special Accounts",
        "note": "",
        "price": 10000000, // 10 USD
        "ast": {
            "type": "function",
            "name": "in_list",
            "arguments": [
                {
                    "type": "variable",
                    "name": "account"
                },
                {
                    "type": "value",
                    "value_type": "binary[]",
                    "value": [
                        "0x...",
                        "0x...",
                        ...
                    ]
                },
            ]
        }
    },
]
```

## AST Parsing Example Code


```typescript
type SubAccountRule = {
    index: number,
    name: string,
    note: string,
    price: number,
    ast: Expression,
}

enum ExpressionType {
    Operator = 'operator',
    Function = 'function',
    Variable = 'variable',
    Value = 'value',
}

enum VairableName {
    Account = 'account',
    AccountChars = 'account_chars',
    AccountLength = 'account_length',
}

enum ValueType {
    Bool = 'bool',
    Uint8 = 'uint8',
    Uint32 = 'uint32',
    Uint64 = 'uint64',
    Binary = 'binary',
    String = 'string',
    StringArray = 'string[]',
    CharsetType = 'charset_type',
}

type Expression = OperatorExpr | FunctionExpr | VariableExpr | ValueExpr

type OperatorExpr = {
    type: ExpressionType.Operator,
    symbol: string,
    expressions: Expression[],
}

type FunctionExpr = {
    type: ExpressionType.Function,
    name: string,
    arguments: Expression[],
}

type VariableExpr = {
    type: ExpressionType.Variable,
    name: string,
}

type ValueExpr = {
    type: ExpressionType.Value,
    value_type: ValueType,
    value: boolean | number | string | string[],
}

function handleExpression(expr: Expression, account_chars): ValueExpr {
    let ret;
    switch(expr.type) {
        case 'operator':
            ret = handleOperator(expr, account_chars)
            break
        case 'function':
            ret = handleFunction(expr, account_chars)
            break
        case 'variable':
            ret = handleVariable(expr, account_chars)
            break
        case 'value':
            ret = expr
            break
        default:
            throw new Error('Unimplement expression found')
    }
    return ret
}

function handleOperator(operator: OperatorExpr, account_chars): ValueExpr {
    let ret: ValueExpr;
    switch(operator.symbol) {
        case "and":
            ret = AndOperator(operator.expressions, account_chars)
            break
        case "or":
            ret = OrOperator(operator.expressions, account_chars)
            break
        case "==":
            ret = EqualOperator(operator.expressions, account_chars)
            break
        // TODO more operator handler functions here ...
        default:
            throw new Error('Unimplement operator found')
    }

    return ret
}

function AndOperator(expressions: Expression[], account_chars): ValueExpr {
    for (let expr of expressions) {
        let ret = handleExpression(expr, account_chars)
        if (ret.type == 'value' && ret.value_type == 'bool') {
            if (ret.value) {
                continue
            } else {
                return {
                    type: ExpressionType.Value,
                    value_type: ValueType.Bool,
                    value: false
                }
            }
        } else {
            throw new Error('Expression type error, expected boolean')
        }
    }

    return {
        type: ExpressionType.Value,
        value_type: ValueType.Bool,
        value: true
    }
}

function OrOperator(expressions: Expression[], account_chars): ValueExpr {
    for (let expr of expressions) {
        let ret = handleExpression(expr, account_chars)
        if (ret.type == 'value' && ret.value_type == 'bool') {
            if (ret.value) {
                return {
                    type: ExpressionType.Value,
                    value_type: ValueType.Bool,
                    value: true
                }
            }
        } else {
            throw new Error('Expression type error, expected boolean')
        }
    }

    return {
        type: ExpressionType.Value,
        value_type: ValueType.Bool,
        value: false
    }
}

function EqualOperator(expressions: Expression[], account_chars): ValueExpr {
    if (expressions.length !== 2) {
        throw new Error('The == operator must accept 2 expressions')
    }

    let left = handleExpression(expressions[0], account_chars)
    if (![ValueType.Uint8, ValueType.Uint32, ValueType.Uint64].includes(left.value_type)) {
        throw new Error('The final value of == operator expression must be uint type')
    }

    let right = handleExpression(expressions[1], account_chars)
    if (![ValueType.Uint8, ValueType.Uint32, ValueType.Uint64].includes(right.value_type)) {
        throw new Error('The final value of == operator expression must be uint type')
    }

    return {
        type: ExpressionType.Value,
        value_type: ValueType.Bool,
        value: left.value === right.value
    }
}

function handleFunction(functionExpr: FunctionExpr, account_chars): ValueExpr {
    let ret: ValueExpr;
    switch(functionExpr.name) {
        case "include_chars":
            ret = includeChars(functionExpr.arguments, account_chars)
            break
        case "only_include_charset":
            ret = includeCharset(functionExpr.arguments, account_chars)
            break
        // TODO more operator handler functions here ...
        default:
            throw new Error('Unimplement function found')
    }

    return ret
}

function includeChars(args: Expression[], account_chars): ValueExpr {
    // TODO to be implement ...
    return {
        type: ExpressionType.Value,
        value_type: ValueType.Bool,
        value: true
    }
}

function includeCharset(args: Expression[], account_chars): ValueExpr {
    // TODO to be implement ...
    return {
        type: ExpressionType.Value,
        value_type: ValueType.Bool,
        value: true
    }
}

function handleVariable(variable: VariableExpr, account_chars): ValueExpr {
    switch(variable.name) {
        case 'account':
            // TODO to be implement ...
            return {
                type: ExpressionType.Value,
                value_type: ValueType.Uint32,
                value: 4
            }
        case 'account_chars':
            // TODO to be implement ...
            return {
                type: ExpressionType.Value,
                value_type: ValueType.StringArray,
                value: [ '2️⃣', '0️⃣', '7️⃣', '7️⃣' ]
            }
        case 'account_length':
            // TODO to be implement ...
            return {
                type: ExpressionType.Value,
                value_type: ValueType.Uint8,
                value: 4
            }
        default:
            throw new Error('Unsupported variable')
    }
}

function findSubAccountRule(subAccountRules: SubAccountRule[], account_chars): SubAccountRule {
    for (let subAccountRule of subAccountRules) {
        let val = handleExpression(subAccountRule.ast, account_chars)
        if (val.value_type !== ValueType.Bool) {
            throw new Error('AST returned invalid value')
        } else{
            if (val.value) {
                return subAccountRule
            } else {
                continue
            }
        }
    }

    throw new Error('Can not find any price for the account')
}

const subAccountRules: SubAccountRule[] = [
    {
        "name": "4 位 emoji 账户",
        "note": "",
        "price": 100000000, // 100 USD
        "ast": {
            "type": ExpressionType.Operator,
            "symbol": "and",
            "expressions": [
                {
                    "type": ExpressionType.Operator,
                    "symbol": "==",
                    "expressions": [
                        {
                            "type": ExpressionType.Variable,
                            "name": "account_length"
                        },
                        {
                            "type": ExpressionType.Value,
                            "value_type": ValueType.Uint8,
                            "value": 4
                        }
                    ]
                },
                {
                    "type": ExpressionType.Function,
                    "name": "only_include_charset",
                    "arguments": [
                        {
                            "type": ExpressionType.Variable,
                            "name": "account_charts"
                        },
                        {
                            "type": ExpressionType.Value,
                            "value_type": ValueType.CharsetType,
                            "value": "Emoji"
                        }
                    ]
                }
            ]
        }
    },
]

const accountChars = [
    {
        char_set: 'emoji',
        chars: '2️⃣'
    },
    {
        char_set: 'emoji',
        chars: '0️⃣'
    },
    {
        char_set: 'emoji',
        chars: '7️⃣'
    },
    {
        char_set: 'emoji',
        chars: '7️⃣'
    },
]

let subAccountRule = findSubAccountRule(subAccountRules, accountChars)

console.log(subAccountRule)
```
