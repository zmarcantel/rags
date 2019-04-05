#[cfg(test)]
mod lists {
    use crate::*;

    #[test]
    fn basic() {
        let mut flags: bool = false;
        let mut file: String = "".to_string();

        Parser::from_strings(string_vec!("argv[0]", "-s", "--long", "my_file"))
            .short_flag('s',   "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .positional("file", "", &mut file, false)
                .expect("could not crete positional")
        ;

        assert!(file == "my_file", "did not pick up positional");
    }

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
    fn interleaved() {
        let mut flags: bool = false;
        let mut args: String = "".to_string();
        let mut count: usize = 0;
        let mut file: String = "".to_string();

        let argv = string_vec!("argv[0]",
            "-s", "--long", "my_file", "-f=foo", "-o", "other", "-v"
        );
        Parser::from_strings(argv)
            .short_flag('s',   "check short only", &mut flags, false)
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
            .short_flag('s',   "check short only", &mut flags, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut flags, false)
                .expect("bad long mode")
            .positional("file", "", &mut file, false)
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

