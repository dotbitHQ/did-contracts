macro_rules! load_template {
    (&mut $context:expr, &mut $parser:expr, $file:expr) => {{
        $context = Context::default();
        $parser = TemplateParser::from_file(&mut $context, format!("./templates/{}", $file))
            .expect("Init template parser failed.");

        // parse transaction template
        $parser.parse();
    }};
}
