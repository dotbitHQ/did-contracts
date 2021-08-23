#[macro_export]
macro_rules! typed_data_v4 {
    (@value {$( $type_name:ident: $tail:tt ),+}) => {{
        let mut types = serde_json::Map::new();
        $(
            types.insert(alloc::string::String::from(stringify!($type_name)), serde_json::Value::from(typed_data_v4!(@value $tail)));
        )+
        types
    }};
    (@value [$( $field_name:ident: $field_type:expr ),+]) => {
        alloc::vec![
            $(
                $crate::types::DomainTypeField {
                    name: alloc::string::String::from(stringify!($field_name)),
                    type_: alloc::string::String::from($field_type),
                },
            )+
        ]
    };
    (@value $value:expr) => {
        $value
    };
    ({ types: $types:tt, primaryType: $primary_type:expr, domain: $domain:tt, message: $message:tt }) => {
        $crate::types::TypedDataV4::new(
            typed_data_v4!(@value $types),
            $primary_type,
            typed_data_v4!(@value $domain),
            typed_data_v4!(@value $message)
        )
    };
}
