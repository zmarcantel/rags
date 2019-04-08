#[cfg(test)]
mod flags {
    use crate::*;

    #[test]
    fn basic() {
        let mut debug_mode_short: bool = false;
        let mut debug_mode_long: bool = false;

        Parser::from_strings(string_vec!("argv[0]", "-s", "--long"))
            .short_flag('s',   "check short only", &mut debug_mode_short, false)
                .expect("bad short mode")
            .long_flag("long", "check long only",  &mut debug_mode_long, false)
                .expect("bad long mode")
        ;

        assert!(debug_mode_short, "did not set with short flag");
        assert!(debug_mode_long, "did not set with long flag");
    }

    #[test]
    fn inverted() {
        let mut debug_mode_short: bool = true;
        let mut debug_mode_long: bool = true;

        Parser::from_strings(string_vec!("argv[0]", "-s", "--long"))
            .short_flag('s',   "check short only", &mut debug_mode_short, true).expect("bad short mode")
            .long_flag("long", "check long only",  &mut debug_mode_long, true).expect("bad long mode")
        ;

        assert!(debug_mode_short == false, "did not invert with short flag");
        assert!(debug_mode_long == false, "did not invert with long flag");
    }

    #[test]
    fn within_run() {
        let expect_count: usize = 8;
        let mut count: usize = 0;
        let mut other: usize = 0;
        let mut debug: bool = false;
        Parser::from_strings(string_vec!("argv[0]", "-vxvxvDxvx", "-vxvxvxvx"))
            .count('v', "verbose", "increase verbosity", &mut count, 1)
                .expect("bad count parse")
            .count('x', "extreme", "do something different", &mut other, 1)
                .expect("bad count parse")
            .short_flag('D', "flag something as true", &mut debug, false)
                .expect("bad flag parse")
        ;

        assert!(count == expect_count,
            "unexpected count value {}, wanted {}", count, expect_count);
        assert!(other == expect_count,
            "unexpected other value {}, wanted {}", other, expect_count);
        assert!(debug, "expected flag to be caugh within the run, got {}", debug);
    }
}

