extern crate rags;
use rags::argparse;

static LONG_DESC: &'static str =
"This example aims to show beginner to intermediate options on the parser
as well as good practices.

As such, the usefulness of the binary is minimal but it will show you how
an application should be structured, options passed, errors handled, and
using parser state to control execution flow (print_help+exit, subcommands, etc).";

#[derive(Debug)]
pub struct Options {
    file: String,
    debug: bool,
    verbosity: usize,

    subcmds: Vec<String>,

    build_release: bool,
    build_link: Vec<String>,

    dry_run: bool,
}
impl Options {
    pub fn new() -> Options {
        Options {
            file: "default.file".to_string(),
            debug: false,
            verbosity: 0,

            subcmds: vec!(),

            build_release: false,
            build_link: vec!(),

            dry_run: false,
        }
    }
}

fn handle_args(parser: &mut rags::Parser, opts: &mut Options) -> Result<(), rags::Error> {
    parser
        .app_long_desc(LONG_DESC)
        .group("logging", "adjust logging output")?
            .flag('D', "debug", "enter debug mode", &mut opts.debug, false)?
            .count('v', "verbose", "increase vebosity (can be given multiple times)",
                &mut opts.verbosity, 1)?
            .done()?
        .subcommand("build", "build a target", &mut opts.subcmds, None)?
            .arg('f', "file", "file to build", &mut opts.file, Some("FILE"), true)?
            .list('l', "lib", "libraries to link", &mut opts.build_link, Some("LIB"), false)?
            .long_flag("release", "do a release build", &mut opts.build_release, false)?
            .done()?
        .subcommand("clean", "clean all build artifacts", &mut opts.subcmds, None)?
            .flag('p', "print-only", "print what files would be cleaned, but do not clean",
                &mut opts.dry_run, false)?
            .done()?
    ;

    Ok(())
}

fn main() {
    let mut opts = Options::new();
    let mut parser = argparse!();
    match handle_args(&mut parser, &mut opts) {
        Ok(_) => {}
        Err(e) => {
            println!("");
            println!("ERROR: {}", e);
            println!("");
            parser.print_help();
            std::process::exit(1);
        }
    }

    if parser.wants_help() {
        parser.print_help();
    } else {
        println!("final config: {:?}", opts);
    }
}
