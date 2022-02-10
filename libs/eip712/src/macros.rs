#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "std")]
        println!($($arg)*)
    };
}

#[macro_export]
macro_rules! typed_data_v4 {
    (@types_1 { $( $type_name:ident: $tail:tt ),+ }) => {{
        let mut types = alloc::collections::BTreeMap::new();
        $(
            types.insert(alloc::string::String::from(stringify!($type_name)), typed_data_v4!(@types_2 $tail));
        )+
        types
    }};
    (@types_2 { $( $type_name:ident: $tail:expr ),+ }) => {{
        let mut types = alloc::vec::Vec::new();
        $(
            types.push((
                alloc::string::String::from(stringify!($type_name)),
                alloc::string::String::from($tail)
            ));
        )+
        types
    }};
    (@object {$( $key:ident: $val:expr ),+}) => {{
        let mut keys = alloc::vec::Vec::new();
        let mut object = alloc::collections::BTreeMap::new();
        $(
            keys.push(alloc::string::String::from(stringify!($key)));
            object.insert(alloc::string::String::from(stringify!($key)), $crate::eip712::Value::String(alloc::string::String::from($val)));
        )+
        $crate::eip712::Value::Object((keys, object))
    }};
    (@object $val:expr) => { $val };
    (@array [$( $item:tt ),+]) => {{
        let mut arr = alloc::vec::Vec::new();
        $(
            arr.push(typed_data_v4!(@object $item));
        )+
        $crate::eip712::Value::Array(arr)
    }};
    (@array $val:expr) => { $val };
    (@domain {
        $key_chain_id:ident: $val_chain_id:expr,
        $key_name:ident: $val_name:expr,
        $key_verifying_contract:ident: $val_verifying_contract:expr,
        $key_version:ident: $val_version:expr
    }) => {{
        let mut keys = alloc::vec::Vec::new();
        keys.push(alloc::string::String::from(stringify!($key_chain_id)));
        keys.push(alloc::string::String::from(stringify!($key_name)));
        keys.push(alloc::string::String::from(stringify!($key_verifying_contract)));
        keys.push(alloc::string::String::from(stringify!($key_version)));

        let mut domain = alloc::collections::BTreeMap::new();
        domain.insert(alloc::string::String::from(stringify!($key_chain_id)), $crate::eip712::Value::Uint256(alloc::string::String::from($val_chain_id)));
        domain.insert(alloc::string::String::from(stringify!($key_name)), $crate::eip712::Value::String(alloc::string::String::from($val_name)));
        domain.insert(alloc::string::String::from(stringify!($key_verifying_contract)), $crate::eip712::Value::Address(alloc::string::String::from($val_verifying_contract)));
        domain.insert(alloc::string::String::from(stringify!($key_version)), $crate::eip712::Value::String(alloc::string::String::from($val_version)));

        $crate::eip712::Value::Object((keys, domain))
    }};
    (@message {
        $key_das_message:ident: $val_das_message:expr,
        $key_inputs_capacity:ident: $val_inputs_capacity:expr,
        $key_outputs_capacity:ident: $val_outputs_capacity:expr,
        $key_fee:ident: $val_fee:expr,
        $key_action:ident: $val_action:tt,
        $key_inputs:ident: $val_inputs:tt,
        $key_outputs:ident: $val_outputs:tt,
        $key_digest:ident: $val_digest:expr
    }) => {{
        let mut keys = alloc::vec::Vec::new();
        keys.push(alloc::string::String::from(stringify!($key_das_message)));
        keys.push(alloc::string::String::from(stringify!($key_action)));
        keys.push(alloc::string::String::from(stringify!($key_inputs_capacity)));
        keys.push(alloc::string::String::from(stringify!($key_outputs_capacity)));
        keys.push(alloc::string::String::from(stringify!($key_fee)));
        keys.push(alloc::string::String::from(stringify!($key_inputs)));
        keys.push(alloc::string::String::from(stringify!($key_outputs)));

        let mut message = alloc::collections::BTreeMap::new();
        message.insert(alloc::string::String::from(stringify!($key_das_message)), $crate::eip712::Value::String(alloc::string::String::from($val_das_message)));
        message.insert(alloc::string::String::from(stringify!($key_action)), typed_data_v4!(@object $val_action));
        message.insert(alloc::string::String::from(stringify!($key_inputs_capacity)), $crate::eip712::Value::String(alloc::string::String::from($val_inputs_capacity)));
        message.insert(alloc::string::String::from(stringify!($key_outputs_capacity)), $crate::eip712::Value::String(alloc::string::String::from($val_outputs_capacity)));
        message.insert(alloc::string::String::from(stringify!($key_fee)), $crate::eip712::Value::String(alloc::string::String::from($val_fee)));
        message.insert(alloc::string::String::from(stringify!($key_inputs)), typed_data_v4!(@array $val_inputs));
        message.insert(alloc::string::String::from(stringify!($key_outputs)), typed_data_v4!(@array $val_outputs));
        message.insert(alloc::string::String::from(stringify!($key_digest)), $crate::eip712::Value::Byte32(alloc::string::String::from($val_digest)));

        $crate::eip712::Value::Object((keys, message))
    }};
    ({ types: $types:tt, primaryType: $primary_type:expr, domain: $domain:tt, message: $message:tt }) => {{
        $crate::eip712::TypedDataV4 {
            types: typed_data_v4!(@types_1 $types),
            primary_type: Value::String(String::from($primary_type)),
            domain: typed_data_v4!(@domain $domain),
            message: typed_data_v4!(@message $message),
        }
    }};
}
