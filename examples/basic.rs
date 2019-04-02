extern crate rags;
use rags::argparse;

#[derive(Debug)]
pub struct Options {
    file: String,
    debug: bool,
    verbosity: usize,

    subcmds: Vec<String>,

    build_release: bool,
    build_link: Vec<String>,
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
        }
    }
}

fn handle_args(parser: &mut rags::Parser, opts: &mut Options) -> Result<(), rags::Error> {
    parser
        .group("logging", "adjust logging output")?
            .flag('D', "debug", "enter debug mode", &mut opts.debug, false)?
            .count('v', "verbose", "increase vebosity (can be given multiple times)",
                &mut opts.verbosity, 1)?
            .done()?
        .subcommand("build", "build a target", &mut opts.subcmds)?
            .arg('f', "file", "file to build", &mut opts.file, Some("FILE"), true)?
            .list('l', "lib", "libraries to link", &mut opts.build_link, Some("LIB"), false)?
            .long_flag("release", "do a release build", &mut opts.build_release, false)?
            .done()?
    ;

    parser.print_help();

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
    println!("final config: {:?}", opts);
}
