macro_rules! parse_template {
    ($file:expr) => {{
        let mut parser =
            TemplateParser::from_file(Context::default(), format!("./templates/{}", $file))
                .expect("Init template parser failed.");

        parser.parse();

        parser
    }};
}

macro_rules! test_with_template {
    ($test_name:ident, $file:expr) => {
        #[test]
        fn $test_name() {
            let mut parser = parse_template!($file);
            let cycles = parser
                .execute_tx_directly()
                .expect("Transaction verification should pass.");

            println!("{} costs: {} cycles", stringify!($test_name), cycles);
        }
    };
}
