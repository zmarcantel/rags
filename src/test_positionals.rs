#[cfg(test)]
mod named {
    use crate::*;

    #[test]
    fn basic() {
        let mut flags: bool = false;
        let mut file: String = "".to_string();

        Parser::from_strings(string_vec!("argv[0]", "-s", "--long", "my_file"))
            .short_flag('s', "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .positional("file", "", &mut file, false)
                .expect("could not crete positional")
        ;

        assert!(file == "my_file", "did not pick up positional");
    }

    #[test]
    fn interleaved() {
        let mut flags: bool = false;
        let mut args: String = "".to_string();
        let mut count: usize = 0;
        let mut file: String = "".to_string();

        let argv = string_vec!("argv[0]",
            "-s", "--long", "my_file", "-f=foo", "-o", "other", "-v"
        );
        Parser::from_strings(argv)
            .short_flag('s', "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .short_arg('f', "just need an arg",  &mut args, None, false)
                .expect("arg")
            .short_arg('o', "just need another arg",  &mut args, None, false)
                .expect("arg")
            .short_count('v', "just need a count",  &mut count, 1)
                .expect("count")
            .positional("file", "", &mut file, false)
                .expect("could not crete positional")
        ;

        assert!(file == "my_file", "did not pick up positional");
    }

    #[test]
    fn missing() {
        let mut flags: bool = false;
        let mut file: String = "".to_string();

        let mut parse = Parser::from_strings(string_vec!("argv[0]", "-s", "--long"));
        let result = parse
            .short_flag('s', "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .positional("file", "", &mut file, true)
        ;

        match &result {
            Ok(_) => {
                assert!(false, "did not receive missing positional error");
            }
            Err(e) => {
                match e {
                    Error::MissingPositional(_) => {}
                    _ => {
                        assert!(false, "got wrong error: {}", e);
                    }
                }
            }
        }
        assert!(file.is_empty(), "did not pick up positional");
    }
}


#[cfg(test)]
mod lists {
    use crate::*;

    #[test]
    fn variadic() {
        let mut flags: bool = false;
        let mut files: Vec<String> = vec!();

        let argv = string_vec!("argv[0]", "file1", "-s", "file2", "--long", "file3");
        Parser::from_strings(argv)
            .short_flag('s', "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .positional_list("file", "", &mut files, false)
                .expect("could not crete positional")
        ;

        assert!(files.len() == 3, "expected 3 files, got {}", files.len());
        assert!(files[0] == "file1");
        assert!(files[1] == "file2");
        assert!(files[2] == "file3");
    }

    #[test]
    fn multi_variadic() {
        let mut flags: bool = false;
        let mut files: Vec<String> = vec!();

        let argv = string_vec!("argv[0]", "file1", "-s", "file2", "--long", "file3");
        let mut parser = Parser::from_strings(argv);
        let result = parser
            .short_flag('s', "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .positional_list("file", "", &mut files, false)
                .expect("could not create first positional list")
            .positional_list("should_error", "", &mut files, false)
        ;

        match result {
            Ok(_) => { assert!(false, "did not receive errror"); }
            Err(e) => match e {
                Error::MultipleVariadic(_) => {}
                _ => {
                    assert!(false, "received incorrect error: {}", e);
                }
            }
        }

    }


    #[test]
    fn collects_after_argstop() {
        let mut flags: bool = false;
        let mut name: String = String::new();
        let mut subbies: Vec<String> = vec!();
        let mut post_stop: Vec<String> = vec!();

        let argv = string_vec!("argv[0]", "foo", "-s", "-n=argstop", "--", "--long", "foo");
        let mut parser = Parser::from_strings(argv);
        parser
            .short_flag('s', "check short", &mut flags, false)
                .expect("bad short mode")
            .subcommand("foo", "blah", &mut subbies, None)
                .expect("could not create positional")
                .arg('n', "name", "test stuff", &mut name, None, true)
                    .expect("failed to parse name")
                .positional_list("file", "", &mut post_stop, false)
                    .expect("could not create positional list")
                .done().expect("failed to close subcommand scope")

        ;

        assert!(flags, "did not set short flag");
        assert_eq!(name, "argstop", "did not set name");
        assert_eq!(subbies.len(), 1, "incorrect subcommands length");
        assert_eq!(subbies[0], "foo", "incorrect subcommand");
        assert_eq!(post_stop.len(), 2, "incorrect post-argstop length");
        assert_eq!(post_stop[0], "--long", "incorrect first post-stop arg");
        assert_eq!(post_stop[1], "foo", "incorrect second post-stop arg");
    }
}

