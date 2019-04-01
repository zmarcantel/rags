extern crate rags;

#[derive(Debug)]
pub struct Options {
    file: String,
    debug: bool,
    verbosity: usize,
}
impl Options {
    pub fn new() -> Options {
        Options {
            file: "default.file".to_string(),
            debug: false,
            verbosity: 0,
        }
    }
}

fn handle_args(opts: &mut Options) -> Result<(), rags::ParseError> {
    rags::Parser::from_args()
        .arg('f', "file", "file to cat to stdout", &mut opts.file)?
        .flag('D', "debug", "enter debug mode", &mut opts.debug, false)?
        .count('v', "verbose", "increase vebosity (can be given multiple times)",
            &mut opts.verbosity, 1)?
    ;

    Ok(())
}

fn main() {
    let mut opts = Options::new();
    match handle_args(&mut opts) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
    println!("final config: {:?}", opts);
}
