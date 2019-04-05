#[cfg(test)]
mod args {
    use crate::*;

    #[test]
    fn basic() {
        let mut file: String = "default.file".to_string();
        let mut short: String = "".to_string();
        let mut long: String = "".to_string();

        let args = string_vec!("argv[0]", "-f", "foo.bar", "-s", "foo", "--long", "bar");
        Parser::from_strings(args)
            .arg('f', "file", "file to handle", &mut file, None, false)
                .expect("failed to parse file argument")
            .short_arg('s', "a short arg", &mut short, None, false)
                .expect("failed to parse short arg")
            .long_arg("long", "a long arg", &mut long, None, false)
                .expect("failed to parse short arg")
        ;

        assert!(file == "foo.bar", "got unexpected 'file' value: {}", file);
        assert!(short == "foo", "got unexpected 'short' value: {}", short);
        assert!(long == "bar", "got unexpected 'long' value: {}", long);
    }

    #[test]
    fn with_eq() {
        let mut file: String = "default.file".to_string();
        let mut short: u16 = 50;
        let mut long: usize = 100;

        let args = string_vec!("argv[0]", "-f=foo.bar", "-s=17", "--long", "10");
        Parser::from_strings(args)
            .arg('f', "file", "file to handle", &mut file, None, false)
                .expect("failed to parse file argument")
            .short_arg('s', "a short arg", &mut short, None, false)
                .expect("failed to parse short arg")
            .long_arg("long", "a long arg", &mut long, None, false)
                .expect("failed to parse short arg")
        ;

        assert!(file == "foo.bar", "got unexpected 'file' value: {}", file);
        assert!(short == 17, "got unexpected 'short' value: {}", short);
        assert!(long == 10, "got unexpected 'long' value: {}", long);
    }
}

