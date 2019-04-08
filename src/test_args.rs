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

    #[test]
    fn runs_at_end() {
        let expect_count: usize = 8;
        let mut count: usize = 0;
        let mut other: usize = 0;
        let mut file: String = "".to_string();
        Parser::from_strings(string_vec!("argv[0]", "-vxvxvxvxf", "foo.file", "-vxvxvxvx"))
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
            .count('x', "extreme", "do something different", &mut other, 1)
                .expect("bad count parse")
            .short_arg('f', "file to consider", &mut file, None, false)
                .expect("bad arg parse")
        ;

        assert!(count == expect_count,
            "unexpected count value {}, wanted {}", count, expect_count);
        assert!(other == expect_count,
            "unexpected other value {}, wanted {}", other, expect_count);
        assert!(file == "foo.file", "expected file to be 'foo.file' but got {}", file);
    }

    #[test]
    fn runs_must_be_at_end() {
        let mut count: usize = 0;
        let mut other: usize = 0;
        let mut file: String = "".to_string();
        let args = string_vec!("argv[0]", "-vxvxvfxvx", "foo.file", "-vxvxvxvx");
        let mut parser = Parser::from_strings(args);
        let result = parser
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
            .count('x', "extreme", "do something different", &mut other, 1)
                .expect("bad count parse")
            .short_arg('f', "file to consider", &mut file, None, false)
        ;

        match result {
            Ok(_) => { assert!(false, "expected error about args being at end of runs"); }
            Err(e) => { match e {
                Error::ValuedArgInRun(_, _) => { /* did what we expect */ }
                _ => {
                    assert!(false, "unexpected error: {:?}", e);
                }
            }}
        }
    }
}

