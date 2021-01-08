macro_rules! load_template {
    (&mut $context:expr, &mut $parser:expr, $file:expr) => {{
        $context = Context::default();
        $parser = TemplateParser::new(&mut $context, include_str!($file))
            .expect("Init template parser failed.");

        // parse transaction template
        $parser.parse();
    }};
}
