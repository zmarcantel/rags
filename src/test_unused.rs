#[cfg(test)]
mod unused {
    use crate::*;

    #[test]
    fn unknown_arg() {
        let mut flag: bool = false;
        let mut count: usize = 0;

        let args = string_vec!("argv[0]", "-f", "-cccc", "--file", "boo.berry");
        let mut parser = Parser::from_strings(args);
        parser
            .short_flag('f', "flag that does something", &mut flag, false)
                .expect("flag parse error")
            .short_count('c', "count that does something", &mut count, 1)
                .expect("count parse error")
        ;

        assert!(flag == true, "expected flag to be true");
        assert!(count == 4, "expected count to be 4 but got {}", count);

        let unused = parser.unused();
        assert!(unused.len() == 2, "expected 2 unused, got {}", unused.len());

        assert!(unused[0].arg == "--file",
            "exepcted unused[0] to be '--file', got '{}'", unused[0].arg);
        assert!(unused[0].looks_like == LooksLike::LongArg,
            "exepcted unused[0] to look like long arg, got '{}'", unused[0].looks_like);

        assert!(unused[1].arg == "boo.berry",
            "exepcted unused[1] to be 'boo.berry', got '{}'", unused[1].arg);
        assert!(unused[1].looks_like == LooksLike::Positional,
            "exepcted unused[1] to look like positional, got '{}'", unused[1].looks_like);
    }


    #[test]
    fn unhandled_positional() {
        let mut flag: bool = false;
        let mut count: usize = 0;

        let args = string_vec!("argv[0]", "-f", "-cccc", "boo.berry");
        let mut parser = Parser::from_strings(args);
        parser
            .short_flag('f', "flag that does something", &mut flag, false)
                .expect("flag parse error")
            .short_count('c', "count that does something", &mut count, 1)
                .expect("count parse error")
        ;

        assert!(flag == true, "expected flag to be true");
        assert!(count == 4, "expected count to be 4 but got {}", count);

        let unused = parser.unused();
        assert!(unused.len() == 1, "expected only 1 unused, got {}", unused.len());
        assert!(unused[0].arg == "boo.berry", "got unexpected unused: {}", unused[0].arg);
    }
}
