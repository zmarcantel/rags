#[cfg(test)]
mod subcmds {
    use crate::*;

    #[test]
    fn basic() {
        let mut subs: Vec<String> = vec!();

        Parser::from_strings(string_vec!("argv[0]", "run", "until", "midnight", "-vvvvv"))
            .subcommand("build", "do a build", &mut subs, None).expect("bad sub(build)")
                .done().expect("no done on build")
            .subcommand("run", "run a target", &mut subs, None).expect("bad sub(run)")
                .subcommand("until", "run a target until a time", &mut subs, None)
                    .expect("bad sub-sub(until)")
                    .subcommand("midnight", "alias for passing in midnnight", &mut subs, None)
                        .expect("bad sub-sub-sub(midnight)")
                        .done().expect("no done on run-until-midnight")
                    .done().expect("no done on run-until")
                .done().expect("no done on run")
            .subcommand("test", "test a target", &mut subs, None).expect("bad sub(test)")
                .done().expect("no done on test")
        ;

        assert!(subs.len() == 3, "incorrect vector len {}", subs.len());
        assert!(subs[0] == "run", "incorrect vector value[0] {}", subs[0]);
        assert!(subs[1] == "until", "incorrect vector value[1] {}", subs[1]);
        assert!(subs[2] == "midnight", "incorrect vector value[2] {}", subs[2]);
    }

    #[test]
    fn hygiene() {
        let mut subs: Vec<String> = vec!();
        let mut build_file: String = "build".to_string();
        let mut test_file: String = "test".to_string();

        Parser::from_strings(string_vec!("argv[0]", "build", "-f=hahaha.txt"))
            .subcommand("build", "do a build", &mut subs, None)
                .expect("bad sub(build)")
                .arg('f', "file", "file to build", &mut build_file, None, false)
                    .expect("bad build-file")
                .done().expect("no done on build")
            .subcommand("test", "test a target", &mut subs, None).expect("bad sub(test)")
                .arg('f', "file", "file to test", &mut test_file, None, false)
                    .expect("bad test-file")
                .done().expect("no done on test")
        ;

        assert!(subs.len() == 1, "wrong sub count: {}", subs.len());
        assert!(subs[0] == "build", "did not take build path: {}", subs[0]);

        assert!(build_file == "hahaha.txt", "did not set build-file: {}", build_file);
        assert!(test_file == "test", "overwrote test-file: {}", test_file);
    }
}
