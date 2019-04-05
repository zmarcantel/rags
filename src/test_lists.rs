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
}

