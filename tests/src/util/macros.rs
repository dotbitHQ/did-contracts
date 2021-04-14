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

macro_rules! challenge_with_generator {
    ($test_name:ident, $error_code:expr, $generator_fn:expr) => {
        #[test]
        fn $test_name() {
            let generator = $generator_fn;
            let mut parser = TemplateParser::from_data(Context::default(), generator());
            parser.parse();

            let err = parser.execute_tx_directly().expect_err(
                format!("The test should failed with error code: {} | ", $error_code).as_str(),
            );

            let msg = err.to_string();
            println!("Error message: {}", msg);

            let search = format!("ValidationFailure({})", $error_code);
            assert!(
                msg.contains(search.as_str()),
                "The test should failed with error code: {}",
                $error_code
            );
        }
    };
}
