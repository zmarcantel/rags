#[cfg(test)]
mod lists {
    use crate::*;

    #[test]
    fn basic() {
        let mut test_list: Vec<String> = vec!();

        let args = string_vec!("argv[0]", "-f", "foo.bar", "--file", "bar.baz", "-f", "last");
        Parser::from_strings(args)
            .list('f', "file", "add file to list", &mut test_list, None, false).expect("bad list")
        ;

        assert!(test_list.len() == 3, "incorrect vector len {}", test_list.len());
        assert!(test_list[0] == "foo.bar", "incorrect vector value[0] {}", test_list[0]);
        assert!(test_list[1] == "bar.baz", "incorrect vector value[1] {}", test_list[1]);
        assert!(test_list[2] == "last", "incorrect vector value[2] {}", test_list[2]);
    }

    #[test]
    fn inline_eq() {
        let mut test_list: Vec<String> = vec!();

        Parser::from_strings(string_vec!("argv[0]", "-f=foo.bar", "--file=bar.baz", "-f=last"))
            .list('f', "file", "add file to list", &mut test_list, None, false).expect("bad list")
        ;

        assert!(test_list.len() == 3, "incorrect vector len {}", test_list.len());
        assert!(test_list[0] == "foo.bar", "incorrect vector value[0] {}", test_list[0]);
        assert!(test_list[1] == "bar.baz", "incorrect vector value[1] {}", test_list[1]);
        assert!(test_list[2] == "last", "incorrect vector value[2] {}", test_list[2]);
    }


    #[test]
    fn runs_at_end() {
        let expect_count: usize = 8;
        let mut count: usize = 0;
        let mut other: usize = 0;
        let mut files: Vec<String> = vec!();
        let args = string_vec!("argv[0]",
            "-vxvxvxvxf", "foo.file",
            "-vxvxvxvxf", "other.file"
        );
        Parser::from_strings(args)
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
            .count('x', "extreme", "do something different", &mut other, 1)
                .expect("bad count parse")
            .short_list('f', "file to consider", &mut files, None, false)
                .expect("bad list parse")
        ;

        assert!(count == expect_count,
            "unexpected count value {}, wanted {}", count, expect_count);
        assert!(other == expect_count,
            "unexpected other value {}, wanted {}", other, expect_count);

        assert!(files.len() == 2, "expected 2 files, got: {}", files.len());
        assert!(files[0] == "foo.file", "expected files[0] to be 'foo.file' but got {}", files[0]);
        assert!(files[1] == "other.file", "expected files[1] to be 'other.file' but got {}", files[1]);
    }

    #[test]
    fn runs_must_be_at_end() {
        let mut count: usize = 0;
        let mut other: usize = 0;
        let mut files: Vec<String> = vec!();
        let args = string_vec!("argv[0]",
            "-vxvxvxvxf", "foo.file",
            "-vxvxfvxvx", "other.file"
        );
        let mut parser = Parser::from_strings(args);
        let result = parser
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
            .count('x', "extreme", "do something different", &mut other, 1)
                .expect("bad count parse")
            .short_list('f', "file to consider", &mut files, None, false)
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

