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
}

