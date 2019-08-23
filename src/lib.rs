//! # Introduction
//!
//! `rags` is an easy to use argument parsing library that provides pretty help-printing.
//!
//! The library allows defining arguments in the same tree-like manner that users
//! and developers expect. This leads to efficient parsing as we can efficiently
//! eliminate work based on the state of the parsing. Once an argument has been
//! matched it will never be inspected again.
//!
//! `rags` also makes liberal use of the `From<String>` trait so that arguments
//! can be parsed into any complex type. This means, for example, that an argument
//! naming a file can be constructed directly into a struct wrapping `std::fs::File`.
//! This leads to re-usable code between subcommands and developers can spend less time
//! and effort inspecting args.
//!
//! Arguments in the same level (it's tree-like) are parsed in the order in which
//! they are defined. This means that global args are easy and provides both
//! argument and semantic isolation between subcommands. Once a branch in the parse
//! tree is taken (subcommand), the parser will not consider arguments defined "after"
//! that branch in a higher scope. Because nested subcommands always lead to a lower
//! scope, all arguments along that parse path are considered. This leads to 2 basic
//! rules of usage:
//!
//! 1. global arguments should always be declared first
//! 2. positional arguments should be defined within a subcommand scope even if shared
//!    betwwen subcommands
//!
//!
//!
//! # Example Usage
//!
//! Below is an example of usage that tries to capture most features an concepts.
//! While this had to be edited to match Rust's doctest requirements, the examples
//! directory contains examples which follow best practices in a "real" application
//! such as defining descriptions as static, not returning errors from `main`, etc.
//!
//! ```rust
//! #[derive(Debug)]
//! pub struct Options {
//!     debug: bool,
//!     verbosity: usize,
//!
//!     subcmds: Vec<String>,
//!
//!     build_release: bool,
//!     build_link: Vec<String>,
//!     package: String,
//!
//!     dry_run: bool,
//!
//!     initial_file: String,
//!     additional_files: Vec<String>,
//! }
//! impl Default for Options {
//!     fn default() -> Options {
//!         Options {
//!             debug: false,
//!             verbosity: 0,
//!
//!             subcmds: vec!(),
//!
//!             build_release: false,
//!             build_link: vec!(),
//!             package: "main".to_string(),
//!
//!             dry_run: false,
//!
//!             initial_file: "".to_string(),
//!             additional_files: vec!(),
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), rags::Error> {
//!     let long_desc: &'static str =
//!     "This example aims to show beginner to intermediate options on the parser
//!     as well as good practices.
//!
//!     As such, the usefulness of the binary is minimal but it will show you how
//!     an application should be structured, options passed, errors handled, and
//!     using parser state to control execution flow (print_help+exit, subcommands, etc).";
//!
//!
//!     let mut opts = Options::default();
//!     let mut parser = rags::Parser::from_args();
//!     parser
//!         .app_desc("example using most rags features")
//!         .app_long_desc(long_desc)
//!         .group("logging", "adjust logging output")?
//!             .flag('D', "debug", "enter debug mode", &mut opts.debug, false)?
//!             .count('v', "verbose", "increase vebosity (can be given multiple times)",
//!                 &mut opts.verbosity, 1)?
//!             .done()?
//!         .subcommand("build", "build a target", &mut opts.subcmds, None)?
//!             .arg('p', "package", "rename the package", &mut opts.package, Some("PKG"), true)?
//!             .list('l', "lib", "libraries to link", &mut opts.build_link, Some("LIB"), false)?
//!             .long_flag("release", "do a release build", &mut opts.build_release, false)?
//!             .positional("file", "file to build", &mut opts.initial_file, true)?
//!             .positional_list("files", "additional files to build",
//!                 &mut opts.additional_files, false)?
//!             .done()?
//!         .subcommand("clean", "clean all build artifacts", &mut opts.subcmds, None)?
//!             .flag('p', "print-only", "print what files would be cleaned, but do not clean",
//!                 &mut opts.dry_run, false)?
//!             .done()?
//!     ;
//!
//!     if parser.wants_help() {
//!         parser.print_help();
//!     } else {
//!         println!("final config: {:?}", opts);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//!
//!
//! # Example Help Dialog
//!
//! The above example prints the following under various help requests:
//!
//! ### Root Command Help
//!
//! ```ignore
//! $ rags --help
//! rags - 0.1.0 - example using most rags features
//!
//! usage: rags {subcommand} [-Dv]
//!
//! This example aims to show beginner to intermediate options on the parser
//! as well as good practices.
//!
//! As such, the usefulness of the binary is minimal but it will show you how
//! an application should be structured, options passed, errors handled, and
//! using parser state to control execution flow (print_help+exit, subcommands, etc).
//!
//! subcommands:
//!     build                build a target
//!     clean                clean all build artifacts
//!
//! logging:                 adjust logging output
//!     -D, --debug          enter debug mode [default: false]
//!     -v, --verbose        increase vebosity (can be given multiple times) [default: 0]
//!
//! ```
//!
//!
//!
//! ### Subcommand  Help
//!
//! Notice that in the subcommand help we still see the global arguments.
//!
//! ```ignore
//! $ rags build --help
//! rags build - 0.1.0 - build a target
//!
//! usage: rags build [-Dv -l LIB --release] -p PKG file [files...]
//!
//! logging:                     adjust logging output
//!     -D, --debug              enter debug mode [default: false]
//!     -v, --verbose            increase vebosity (can be given multiple times) [default: 0]
//!
//! options:
//!     -p, --package PKG        rename the package [required, default: main]
//!     -l, --lib LIB            libraries to link
//!         --release            do a release build [default: false]
//!
//! positionals:
//!     file                     file to build [required]
//!     files...                 additional files to build
//!
//! ```

use std::env;
use std::str::FromStr;
use std::string::ToString;
use std::collections::BTreeMap;

extern crate bit_set;

pub mod errors;
pub use errors::*;

mod printer;
use printer::arg_string;

type MatchResult = Result<Option<FoundMatch>, Error>;

#[cfg(test)] mod test_args;
#[cfg(test)] mod test_flags;
#[cfg(test)] mod test_count;
#[cfg(test)] mod test_lists;
#[cfg(test)] mod test_positionals;
#[cfg(test)] mod test_subcmds;
#[cfg(test)] mod test_unused;

/// Helper macro to populate the application name, version, and description
/// from the Cargo manifest. Metadata setter functions can be called multiple
/// times if only some of this information is specified in the manifest.
///
/// If called with no arguments, this is the same as
/// [Parser::from_args](struct.Parser.html#method.from_args).
/// Providing one or more args passes the args to
/// [Parser::from_strings](struct.Parser.html#method.from_strings).
#[macro_export]
macro_rules! argparse {
    () => {{
        let mut p = $crate::Parser::from_args();
        argparse!(p, true)
    }};
    ($args:ident) => {{
        let mut p = $crate::Parser::from_strings($args);
        argparse!(p, true)
    }};
    ($args:expr) => {{
        let mut p = $crate::Parser::from_strings($args);
        argparse!(p, true)
    }};
    ($p:ident, true) => {{
        $p.app_name(env!("CARGO_PKG_NAME"))
            .app_version(env!("CARGO_PKG_VERSION"))
            .app_desc(env!("CARGO_PKG_DESCRIPTION"));
        $p
    }}
}


/// Defines where the value (if any) associated with a given argument is located.
#[derive(Debug)]
enum ValueLocation {
    Unknown,
    HasEqual(usize),
    TakesNext,
}

/// FoundMatch is emitted when we match an argument. This carries all necessary
/// metadata about the argument to be parsed.
struct FoundMatch {
    index: usize,
    run_count: usize,
    value: ValueLocation,
}
impl FoundMatch {
    pub fn new(idx: usize, runs: usize, loc: ValueLocation) -> FoundMatch {
        FoundMatch {
            index: idx,
            run_count: runs,
            value: loc,
        }
    }
}


/// Defines the types of arguments we can handle, and when matched, our best
/// guess as to what kind of arg that is until we can verify with more context.
#[derive(PartialEq)]
pub enum LooksLike {
    ShortArg,
    LongArg,
    Positional,
}
impl std::fmt::Display for LooksLike {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LooksLike::ShortArg => {
                write!(f, "short-arg")
            }
            LooksLike::LongArg => {
                write!(f, "long-arg")
            }
            LooksLike::Positional => {
                write!(f, "positional")
            }
        }
    }
}

/// Unused carries information about arguments which go unmatched.
/// Used both in delineating short-code runs as well as passing back
/// all unmatched arguments to the user (when requested via
/// [Parser::unused](struct.Parser.html#method.unused)).
pub struct Unused {
    pub arg: String,
    pub looks_like: LooksLike,
}
impl Unused {
    pub fn new(value: String) -> Unused {
        let arg_0 = value.chars().nth(0).or(Some('\0')).unwrap();
        let arg_1 = value.chars().nth(1).or(Some('\0')).unwrap();

        let looks_like = if (arg_0 == '-') && (arg_1 == '-') {
            LooksLike::LongArg
        } else if (arg_0 == '-') && (arg_1 != '-') {
            LooksLike::ShortArg
        } else {
            LooksLike::Positional
        };

        Unused {
            arg: value,
            looks_like: looks_like,
        }
    }
}
impl std::fmt::Display for Unused {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.looks_like {
            LooksLike::ShortArg | LooksLike::LongArg => {
                write!(f, "unused or unknown argument: {}", self.arg)
            }
            LooksLike::Positional => {
                write!(f, "unused positional or arg-value: {}", self.arg)
            }
        }
    }
}

/// Parser holds the state required for parsing. The methods provided here
/// define how arguments should be treated as well as where they are constructed.
///
/// Arguments used for help (-h/--help) are already registered, and the boolean
/// for this can be accessed via [Parser::wants_help](#method.wants_help).
///
/// Memory usage of this structure is bounded by the arguments passed as well
/// as a bitset mapping which args have been matched. Rust does not provide
/// an `O(1)` access to the args iterator, thus we store it. This also keeps
/// implementation consistent when using
/// [Parser::from_strings](#method.from_strings)
///
/// This structure can be dropped after handling of args/help are complete.
pub struct Parser {
    args: Vec<String>,
    mask: bit_set::BitSet,
    run_masks: BTreeMap<usize, bit_set::BitSet>,

    walk_depth: usize,
    commit_depth: usize,
    max_depth: usize,
    parse_done: bool,
    curr_group: Option<&'static str>,

    help: bool,
    has_variadic: bool,
    argstop: Option<usize>,
    printer: printer::Printer,
}
impl Parser {
    /// Creates a new parser for the arg strings given.
    pub fn from_strings(input: Vec<String>) -> Parser {
        let argstop = match input.iter().enumerate().find(|(_, a)| a.as_str() == "--") {
            Some((i, _)) => { Some(i) }
            None => { None }
        };
        let count = argstop.unwrap_or(input.len());

        let mut bits = bit_set::BitSet::with_capacity(count);
        // TODO: PR with BitSet::set_all() -- or an inverse iter that iterates all unset
        for i in 1..count {
            bits.insert(i);
        }

        let mut p = Parser{
            args: input,
            mask: bits,
            run_masks: BTreeMap::new(),
            walk_depth: 0,
            commit_depth: 0,
            max_depth: 0,
            parse_done: false,
            curr_group: None,

            help: false,
            has_variadic: false,
            argstop,
            printer: printer::Printer::new(printer::App::empty()),
        };

        let mut wants_help = false;
        p.flag('h', "help", "print this help dialog", &mut wants_help, false)
            .expect("could not handle help flag");
        p.help = wants_help;

        p
    }

    /// Collects the arguments given on the command line and defers to
    /// [Parser::from_strings](#method.from_strings).
    pub fn from_args() -> Parser {
        let args = env::args().collect::<Vec<String>>();
        Parser::from_strings(args)
    }

    /// Unused returns all unmatched args. The [Unused](struct.Unused.html) struct
    /// contains the necessary information to call out unrecognized args or typos in
    /// passed arguments.
    ///
    /// If there is an unused character in a run of shortcodes (e.g. `-abcd`, with `b` unused)
    /// the argument within the [Unused](struct.Unused.html) struct will be prefixed with a dash.
    pub fn unused(&self) -> Vec<Unused> {
        let mut result = vec!();
        for i in self.mask.iter() {
            match self.run_masks.get(&i) {
                None => {}
                Some(mask) => {
                    for m in mask.iter() {
                        let s = format!("-{}", self.args[i].chars().nth(m+1).unwrap());
                        result.push(Unused{
                            arg: s,
                            looks_like: LooksLike::ShortArg,
                        });
                    }
                    continue;
                }
            }

            result.push(Unused::new(self.args[i].clone()));
        }

        result
    }


    //----------------------------------------------------------------
    // help setup
    //----------------------------------------------------------------

    /// Sets the name of the application to be printed in the help dialog.
    /// Printed on the first line of the dialog.
    pub fn app_name<'a>(&'a mut self, name: &'static str) -> &'a mut Parser {
        self.printer.set_name(name);
        self
    }

    /// Sets the description of the application to be printed in the help dialog.
    /// Printed on the first line of the dialog.
    pub fn app_desc<'a>(&'a mut self, desc: &'static str) -> &'a mut Parser {
        self.printer.set_short_desc(desc);
        self
    }

    /// Sets the long-form description of the application to be printed in the
    /// help dialog. Printed after the base application info and usage lines.
    pub fn app_long_desc<'a>(&'a mut self, desc: &'static str) -> &'a mut Parser {
        self.printer.set_long_desc(desc);
        self
    }

    /// Sets the version of the application to be printed in the help dialog.
    /// Printed on the first line of the dialog.
    pub fn app_version<'a>(&'a mut self, vers: &'static str) -> &'a mut Parser {
        self.printer.set_version(vers);
        self
    }

    /// Returns whether the help argument was given and help should be printed.
    /// The help dialog can be printed using [Parser::print_help](#method.print_help).
    pub fn wants_help(&self) -> bool {
        self.help
    }

    /// Prints the help information. If subcommands are provided, the help for
    /// the leaf subcommand is printed.
    pub fn print_help(&self) {
        self.printer.print();
    }


    //----------------------------------------------------------------
    // parse helpers
    //----------------------------------------------------------------

    /// Closes a context opened by calling [Parser::group](#method.group) or
    /// [Parser::subcommand](#method.subcommand).
    pub fn done(&mut self) -> Result<&mut Parser, Error> {
        if self.curr_group.is_some() {
            self.curr_group = None;
            return Ok(self);
        }

        if self.walk_depth == 0 {
            return Err(Error::InvalidState("call to done() at top-level"));
        }

        if (self.walk_depth == self.commit_depth) && ( self.commit_depth == self.max_depth) {
            self.parse_done = true;
        }
        self.walk_depth -= 1;

        Ok(self)
    }

    fn should_ignore(&self, is_subcmd: bool) -> bool {
        if self.parse_done {
            return true;
        }
        if is_subcmd {
            self.walk_depth != (self.commit_depth + 1)
        } else {
            self.walk_depth != self.max_depth
        }
    }

    fn commit_next_level(&mut self) {
        self.commit_depth += 1;
        self.max_depth = std::cmp::max(self.commit_depth, self.max_depth);
    }

    fn walk_next_level(&mut self) {
        self.walk_depth += 1;
    }


    fn handle_run(&mut self, idx: usize, short: char, expect_value: bool) -> MatchResult {
        let arg = &self.args[idx];
        if expect_value && !arg.ends_with(short) {
            return Err(Error::ValuedArgInRun(short, arg.clone()));
        }

        let matches = arg.match_indices(short).map(|(i,_)| i).collect::<Vec<usize>>();
        if matches.is_empty() {
            // no matches here
            return Ok(None);
        }

        // fetch the current mask for this run, or insert a new one
        let runmask = match self.run_masks.get_mut(&idx) {
            Some(mutref) => {
                mutref
            }
            None => {
                let mut bits = bit_set::BitSet::with_capacity(arg.len());
                for i in 1..arg.len() { // skip 0, because we want to skip the leading '-'
                    bits.insert(i);
                }
                self.run_masks.insert(idx, bits);
                self.run_masks.get_mut(&idx).expect("failed to insert run mask")
            }
        };
        if runmask.is_empty() {
            return Ok(None);
        }

        let mut count: usize = 0;
        for i in matches.iter() {
            if runmask.contains(*i) == false { continue; }

            runmask.remove(*i);
            count += 1;
        }
        if count == 0 {
            return Ok(None);
        }

        // when we empty a runmask, we set the "parent" index to be fully used
        if runmask.is_empty() {
            self.mask.remove(idx);
        }

        Ok(Some(FoundMatch::new(idx, count,
            if expect_value {
                ValueLocation::TakesNext
            } else {
                ValueLocation::Unknown
            }
        )))
    }

    fn matches_short(&mut self, idx: usize, short: char, expect_value: bool) -> MatchResult {
        if short == '\0' { return Ok(None); } // no match

        let arg = &self.args[idx];
        if arg.len() < 2 {
            return Ok(None);
        }

        if self.run_masks.contains_key(&idx) {
            return self.handle_run(idx, short, expect_value);
        }


        let mut chars = arg.chars();
        let arg_0 = chars.next().or(Some('\0')).unwrap();
        let arg_1 = chars.next().or(Some('\0')).unwrap();
        let arg_2 = chars.next().or(Some('\0')).unwrap();

        // expect arg[0] to be '-'  -- otherwise, looks like a positional
        // also expect arg[1] NOT to be '-'  -- otherwise, looks like a long
        if arg_0 != '-' {
            return Ok(None);
        } else if arg_1 == '-' {
            return Ok(None);
        }

        // expect arg[1] to be the character we are looking for (so not a long)
        if arg_1 != short {
            // if it is not, but we have something that looks like a run, try that
            if arg.len() > 2 && arg_1 != '-' && arg_2 != '=' {
                return self.handle_run(idx, short, expect_value);
            }
            return Ok(None)
        }

        // if we got here, and the length is 2, we have the base case so just return
        if arg.len() == 2 {
            let has_next = self.mask.contains(idx + 1);
            return if expect_value && has_next {
                Ok(Some(FoundMatch::new(idx, 0, ValueLocation::TakesNext)))
            } else {
                Ok(Some(FoundMatch::new(idx, 0, ValueLocation::Unknown)))
            };
        }

        // if the arg has >2 characters, and arg[2] == '=', then we match and
        // return the '=' offset
        if arg_2 == '=' {
            // return HasEqual regardless of expect_value because errors should be handled there
            // rather than this lower context
            return Ok(Some(FoundMatch::new(idx, 0, ValueLocation::HasEqual(2))));
        }

        // we know the arg has len>=3, arg[2] != '=', so it must be a run
        self.handle_run(idx, short, expect_value)
    }

    fn matches_long(&self, idx: usize, long: &'static str, expect_value: bool) -> MatchResult {
        if long.is_empty() { return Ok(None); }

        let arg = self.args[idx].as_str();
        let end_of_arg = 2 + long.len();

        // not enough string to match
        if arg.len() < end_of_arg {
            return Ok(None);
        }

        // not a long arg
        if &arg[..2] != "--" {
            return Ok(None);
        }

        if &arg[2..end_of_arg] != long {
            return Ok(None);
        }

        // we got exactly what we were looking for, so return
        if arg.len() == end_of_arg {
            let has_next = self.mask.contains(idx + 1);
            return Ok(Some(FoundMatch::new(
                idx, 0,
                if expect_value && has_next {
                    ValueLocation::TakesNext
                } else {
                    ValueLocation::Unknown
                }
            )));
        }

        // we got here, so the string is longer than we expect
        // so check for a '=' trailing and return as such
        if let Some(c) = arg.chars().nth(end_of_arg) {
            if c == '=' {
                // return HasEqual regardless of expect_value because errors should be handled
                // there rather than this lower context
                return Ok(Some(FoundMatch::new(idx, 0, ValueLocation::HasEqual(end_of_arg))));
            }
        }

        // otherwise, no match
        Ok(None)
    }

    fn find_match(&mut self, short: char, long: &'static str, expect_value: bool)
        -> MatchResult
    {
        let mask_view = self.mask.iter().collect::<Vec<usize>>();
        for i in mask_view.iter() {
            match self.matches_short(*i, short, expect_value) {
                Ok(Some(mat)) => {
                    return Ok(Some(mat));
                }
                Ok(None) => {} // no match, so ignore
                Err(e) => { return Err(e); }
            }

            match self.matches_long(*i, long, expect_value) {
                Ok(Some(mat)) => {
                    return Ok(Some(mat));
                }
                Ok(None) => {} // no match, so ignore
                Err(e) => { return Err(e); }
            }
        }
        Ok(None)
    }

    fn find_subcommand(&self, name: &'static str) -> Option<FoundMatch> {
        for i in self.mask.iter() {
            let arg = &self.args[i];
            if arg == name {
                return Some(FoundMatch::new(i, 0, ValueLocation::Unknown));
            }
        }
        None
    }

    // takes index of the arg that matched, not the value to be constructed
    fn construct_arg<T: FromStr>(&mut self,
        info: &FoundMatch,
        short: char, long: &'static str,
        into: &mut T
    ) -> Result<(), Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        match info.value {
            ValueLocation::Unknown => {
                Err(Error::MissingArgValue(short, long))
            }
            ValueLocation::TakesNext => {
                if self.mask.contains(info.index + 1) == false {
                    return Err(Error::MissingArgValue(short, long));
                }
                self.mask.remove(info.index + 1); // mark the argument index as having been used/claimed
                let val = &self.args[info.index + 1];
                *into = T::from_str(val.as_str())
                    .map_err(|e| Error::ConstructionError(short, long, format!("{}", e)))?;
                Ok(())
            }
            ValueLocation::HasEqual(off) => {
                let val = &self.args[info.index][(off+1)..];
                // TODO: val.len() > 1 check + error
                *into = T::from_str(val)
                    .map_err(|e| Error::ConstructionError(short, long, format!("{}", e)))?;
                Ok(())
            }
        }
    }


    //----------------------------------------------------------------
    // arg(s)
    //----------------------------------------------------------------

    /// Registers a long and short code which are expected to be followed by a value.
    /// The associated value can be separated by either a space or an equal sign
    /// (e.g. `--foo=7` or `--foo 7`).
    ///
    /// The type you wish to be parse the arg value into must implement `From<String>`
    /// for construction as well as `ToString` for printing defaults in the help dialog.
    ///
    /// You may provide a label to display next to the argument in the help dialog
    /// (e.g. `-f, --file FILE` where the label here is `FILE`).
    ///
    /// Arguments may additionally be marked as required. If the argument is not provided
    /// when marked as required, this method will return an error which will propogate up
    /// the call stack without parsing further args (fail fast).
    pub fn arg<'a, T: FromStr+ToString>(&'a mut self,
        short: char, long: &'static str, desc: &'static str,
        into: &mut T, label: Option<&'static str>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {

        if self.should_ignore(false) { return Ok(self); }

        // only add help if it is wanted
        if self.wants_help() {
            self.printer.add_arg(
                printer::Argument::new(
                    short, long, desc,
                    label, Some(into.to_string()), required
                ),
                self.curr_group
            )?;
            return Ok(self);
        }

        let found_opt = self.find_match(short, long, true)?;
        if found_opt.is_none() {
            // only required if !help
            if required  && !self.wants_help() {
                return Err(Error::MissingArgument(arg_string(short, long, false)));
            }
            return Ok(self);
        }

        let found = found_opt.unwrap();
        self.mask.remove(found.index);
        self.construct_arg(&found, short, long, into)?;

        Ok(self)
    }

    /// Convenience method for declaring a [Parser::arg](#method.arg) without a long code.
    pub fn short_arg<'a, T: FromStr+ToString>(&'a mut self,
        short: char, desc: &'static str, into: &mut T, label: Option<&'static str>,
        required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        self.arg(short, "", desc, into, label, required)
    }

    /// Convenience method for declaring a [Parser::arg](#method.arg) without a short code.
    pub fn long_arg<'a, T: FromStr+ToString>(&'a mut self,
        long: &'static str, desc: &'static str, into: &mut T, label: Option<&'static str>,
        required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        self.arg('\0', long, desc, into, label, required)
    }



    //----------------------------------------------------------------
    // flag(s)
    //----------------------------------------------------------------

    // TODO: the help flags should be stored on `self` which is why this is
    // a member function. once the flag(s) are configurable we will store them
    // on the parser for this case
    fn is_help_flags(&self, short: char, long: &'static str) -> bool  {
        (short == 'h') || (long == "help")
    }

    /// Flag defines an argument that takes no value, but instead sets a boolean.
    /// Typically when a flag is given the backing bool is set to `true`, however,
    /// the `invert` argument here allows "negative-flags" which instead turn an
    /// option off.
    pub fn flag<'a>(&'a mut self,
        short: char, long: &'static str, desc: &'static str,
        into: &mut bool, invert: bool
    ) -> Result<&'a mut Parser, Error>
    {

        if self.should_ignore(false) { return Ok(self); }

        if self.wants_help() {
            self.printer.add_arg(
                printer::Argument::new(short, long, desc, None, Some(into.to_string()), false),
                self.curr_group
            )?;

            if !self.is_help_flags(short, long) {
                return Ok(self);
            }
        }

        let found_opt = self.find_match(short, long, false)?;
        if found_opt.is_none() {
            return Ok(self);
        }

        let found = found_opt.unwrap();
        self.mask.remove(found.index);

        match found.value {
            ValueLocation::Unknown => {
                *into = !invert;
            }
            ValueLocation::TakesNext => {
                return Err(Error::InvalidInput(short, long, "flag should not have a value"));
            }
            ValueLocation::HasEqual(_) => {
                return Err(Error::InvalidInput(short, long, "flag should not have a value"));
            }
        }

        Ok(self)
    }

    /// Convenience method for declaring a [Parser::flag](#method.flag) without a long code.
    pub fn short_flag<'a>(&'a mut self,
        short: char, desc: &'static str,
        into: &mut bool, invert: bool
    ) -> Result<&'a mut Parser, Error>
    {
        self.flag(short, "", desc, into, invert)
    }

    /// Convenience method for declaring a [Parser::flag](#method.flag) without a short code.
    pub fn long_flag<'a>(&'a mut self,
        long: &'static str, desc: &'static str,
        into: &'a mut bool, invert: bool
    ) -> Result<&'a mut Parser, Error>
    {
        self.flag('\0', long, desc, into, invert)
    }


    //----------------------------------------------------------------
    // count(s)
    //----------------------------------------------------------------

    /// Count (inc|dec)rements a backing numeric type every time the argument is provided.
    /// The classic case is increasing verbosity (e.g. `-v` is 1, `-vvvv` is 4).
    ///
    /// The `step` argument to this method defines what should be added to the target value
    /// every time the arg is seen. You may provide negative numbers to decrement.
    ///
    /// Floating point numeric types are supported, but are atypical.
    pub fn count<'a, T: std::ops::AddAssign + ToString + Clone>(&'a mut self,
        short: char, long: &'static str, desc: &'static str,
        into: &mut T, step: T
    ) -> Result<&'a mut Parser, Error>
    {

        if self.should_ignore(false) { return Ok(self); }

        if self.wants_help() {
            self.printer.add_arg(
                printer::Argument::new(short, long, desc, None, Some(into.to_string()), false),
                self.curr_group
            )?;
            return Ok(self);
        }

        loop { // loop until we get no results back
            let found_opt = self.find_match(short, long, false)?;
            if found_opt.is_none() {
                return Ok(self);
            }

            let found = found_opt.unwrap();
            if found.run_count == 0 { // was not part of a run, remove eniter index
                self.mask.remove(found.index);
            }

            match found.value {
                ValueLocation::Unknown => {
                    if found.run_count == 0 {
                        into.add_assign(step.clone());
                    } else {
                        for _ in 0..found.run_count {
                            into.add_assign(step.clone());
                        }
                    }
                }
                ValueLocation::TakesNext => {
                    return Err(Error::InvalidInput(short, long, "count should not have a value"));
                }
                ValueLocation::HasEqual(_) => {
                    return Err(Error::InvalidInput(short, long, "count should not have a value"));
                }
            }
        }
    }

    /// Convenience method for declaring a [Parser::count](#method.count) without a long code.
    pub fn short_count<'a, T: std::ops::AddAssign + ToString + Clone>(&'a mut self,
        short: char, desc: &'static str,
        into: &mut T, step: T
    ) -> Result<&'a mut Parser, Error>
    {
        self.count(short, "", desc, into, step)
    }

    /// Convenience method for declaring a [Parser::count](#method.count) without a short code.
    pub fn long_count<'a, T: std::ops::AddAssign + ToString + Clone>(&'a mut self,
        long: &'static str, desc: &'static str,
        into: &mut T, step: T
    ) -> Result<&'a mut Parser, Error>
    {
        self.count('\0', long, desc, into, step)
    }


    //----------------------------------------------------------------
    // list(s)
    //----------------------------------------------------------------

    /// List collects values from args and appends them to a vector of the target type.
    ///
    /// Follows the same parsing semantics as [Parser::arg](#method.arg), but appends to
    /// a collection rather a single value. Just as with an arg, the target type must
    /// implement `From<String>` as well as `ToString`. Likewise, the `label` and `required`
    /// arguments to this method work the same.
    pub fn list<'a, T: FromStr + ToString>(&'a mut self,
        short: char, long: &'static str, desc: &'static str,
        into: &mut Vec<T>, label: Option<&'static str>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {

        if self.should_ignore(false) { return Ok(self); }

        if self.wants_help() {
            self.printer.add_arg(
                printer::Argument::new(short, long, desc, label, None, required),
                self.curr_group
            )?;
            return Ok(self);
        }

        let mut found_count = 0;
        loop { // loop until we get no results back
            let found_opt = self.find_match(short, long, true)?;
            if found_opt.is_none() { // TODO: required count -- does this make sense?
                // only requried when !help
                if required && (found_count == 0) && !self.wants_help() {
                    return Err(Error::MissingArgument(arg_string(short, long, false)));
                }
                return Ok(self);
            }
            found_count += 1;

            let found = found_opt.unwrap();
            self.mask.remove(found.index);

            let ctor_result = match found.value {
                ValueLocation::Unknown => {
                    return Err(Error::MissingArgValue(short, long));
                }
                ValueLocation::TakesNext => {
                    self.mask.remove(found.index + 1);
                    let str_val = &self.args[found.index + 1];
                    T::from_str(str_val)
                }
                ValueLocation::HasEqual(eq_idx) => {
                    // index already removed
                    let str_val = &self.args[found.index][eq_idx + 1..];
                    T::from_str(str_val)
                }
            };

            into.push(
                ctor_result
                    .map_err(|e| Error::ConstructionError(short, long, format!("{}", e)))?
            );
        }
    }

    /// Convenience method for declaring a [Parser::list](#method.list) without a long code.
    pub fn short_list<'a, T: FromStr + ToString>(&'a mut self,
        short: char, desc: &'static str,
        into: &mut Vec<T>, label: Option<&'static str>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        self.list(short, "", desc, into, label, required)
    }

    /// Convenience method for declaring a [Parser::list](#method.list) without a short code.
    pub fn long_list<'a, T: FromStr + ToString>(&'a mut self,
        long: &'static str, desc: &'static str,
        into: &mut Vec<T>, label: Option<&'static str>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        self.list('\0', long, desc, into, label, required)
    }



    //----------------------------------------------------------------
    // subcommand(s)
    //----------------------------------------------------------------

    /// Subcommands provide information about what the application should do as well
    /// as giving scope to arguments. This method creates a new context (zero cost)
    /// for which arguments can be defined. By creating a new context we allow for
    /// subcommands to share argument codes with differing meanings. You must close
    /// this context/scope using [Parser::done](#method.done).
    ///
    /// When a subcommand is matched it is appended to a vector. The application is
    /// expected to iterate that vector to determine the correct internal function(s)
    /// to call.
    ///
    /// An optional long description specific to this command can be provided.
    /// The application's long description is not printed in the help dialog when a
    /// subcommand is matched.
    ///
    /// Because subcommands are indistinguishable from positional arguments, all
    /// definitions for positional arguments should be done after defining all subcommands.
    pub fn subcommand<'a, T: FromStr + ToString>(&'a mut self,
        name: &'static str, desc: &'static str, into: &mut Vec<T>,
        long_desc: Option<&'static str>
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        // even if we do not match this subcommand, all parsing until the
        // associated ::done() call happens within the next level so we
        // must move into it unconditionally
        self.walk_next_level();

        if self.should_ignore(true) {
            return Ok(self);
        }

        if self.wants_help() {
            self.printer.add_subcommand(printer::Subcommand::new(name, desc));
            // do not return, subcommands need to continue parsing to set levels
            // and help appropriately
        }

        if name.is_empty() {
            return Err(Error::InvalidState("subcommand(...) given empty name"));
        }

        if let Some(info) = self.find_subcommand(name) {
            self.mask.remove(info.index);
            let arg = &self.args[info.index];
            into.push(
                T::from_str(arg)
                    .map_err(|e| Error::SubConstructionError(name, format!("{}", e)))?
            );

            self.commit_next_level();
            self.printer.new_level(
                name, desc,
                if let Some(d) = long_desc { d } else { "" }
            );
        }

        Ok(self)
    }

    //----------------------------------------------------------------
    // group(s)
    //----------------------------------------------------------------

    /// Group can be used to group various arguments into a named section within
    /// the help dialog. When a help argument is not provided, this is a no-op.
    ///
    /// This method opens a new scope/context which must be closed using
    /// [Parser::done](#method.done). However, no masking of arguments occurs in
    /// this created scope. The only effect a group has is on the printing of args.
    pub fn group<'a>(&'a mut self, name: &'static str, desc: &'static str)
        -> Result<&'a mut Parser, Error>
    {
        if let Some(orig) = self.curr_group {
            return Err(Error::NestedGroup(orig, name));
        }

        if self.should_ignore(false) { return Ok(self); }

        self.curr_group = Some(name);
        if self.wants_help() {
            self.printer.add_group(name, desc)?;
        }
        Ok(self)
    }


    //----------------------------------------------------------------
    // positional(s)
    //----------------------------------------------------------------

    /// Creates a named positional argument. Positionals are taken on an in-order basis
    /// meaning when multiple positionals are defined, the values are constructed in the
    /// order they are provided by the user. This method does not parse anything after
    /// the arg-stop setinel (`--`); see [Parser::positional_list](#method.positional_list).
    ///
    /// You may define as many named positionals as required, but if you simply wish to
    /// capture all positionals, see [Parser::positional_list](#method.positional_list).
    ///
    /// Because positionals are indistinguishable from subcommands, all positionals should
    /// be defined after all subcommands. You can, however, safely define positionals within
    /// a leaf subcommand scope.
    ///
    /// Just as in the base [Parser::arg](#method.arg) case, the target type must implement
    /// both `From<String>` and `ToString`.
    pub fn positional<'a, T: ToString + FromStr>(&'a mut self,
        name: &'static str, desc: &'static str,
        into: &mut T, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        if self.should_ignore(false) { return Ok(self); }

        if self.has_variadic {
            return Err(Error::UnorderedPositionals(name));
        }

        if self.wants_help() {
            let def = into.to_string();
            self.printer.add_positional(printer::Positional::new(
                name, desc, if def.is_empty() { None } else { Some(def) },
                required, false
            ))?;
            return Ok(self);
        }

        let idx = match self.mask.iter().next() {
            Some(i) => { i }
            None => {
                if required {
                    return Err(Error::MissingPositional(name.to_string()));
                } else {
                    return Ok(self);
                }
            }
        };
        let val = &self.args[idx];
        *into = T::from_str(val)
            .map_err(|e| Error::PositionalConstructionError(name, format!("{}", e)))?;

        self.mask.remove(idx);

        Ok(self)
    }

    /// Gathers all unused arguments which are assumed to be positionals. Unused here
    /// does not include short code runs. Unrecognized arguments will also be returned
    /// here as there is mass complexity in determining the difference. For instance,
    /// `-9` is a valid short code flag but also has meaning as a positional.
    ///
    /// All arguments provided after the arg-stop setinel (`--`) will be gathered here.
    /// For example, in `my_app list --foo=7 -- list --help` the trailing `list --help`
    /// will not be parsed as arguments by this parser but instead will be considered
    /// positionals.
    ///
    /// Just as [Parser::list](#method.list) is a vector of [Parser::arg](#method.arg),
    /// this method is a vector of [Parser::positional](#method.positional) sharing a
    /// single name for the set.
    ///
    /// This method may only be called once, or an error will be returned.
    pub fn positional_list<'a, T: ToString + FromStr>(&'a mut self,
        name: &'static str, desc: &'static str,
        into: &mut Vec<T>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        if self.should_ignore(false) { return Ok(self); }

        if self.has_variadic {
            return Err(Error::MultipleVariadic(name));
        } else {
            self.has_variadic = true;
        }

        // TODO: should we print defaults of lists?
        if self.wants_help() {
            self.printer.add_positional(printer::Positional::new(
                name, desc, None, required, true
            ))?;
            return Ok(self);
        }

        let mut found_count: usize = 0;
        // TODO: I hate this, but self.mask.iter() is immut and mask mod is mut....
        let mut found_idxs: Vec<usize> = vec!();
        for i in self.mask.iter() {
            let val = &self.args[i];
            into.push(
                T::from_str(val).map_err(|e|
                    Error::PositionalConstructionError(name, format!("{}", e))
                )?
            );

            found_count += 1;
            found_idxs.push(i);
        }
        for i in found_idxs.iter() {
            self.mask.remove(*i);
        }

        if let Some(stop) = self.argstop {
            for i in (stop+1)..self.args.len() {
                let val = &self.args[i];
                into.push(
                    T::from_str(val).map_err(|e|
                        Error::PositionalConstructionError(name, format!("{}", e))
                    )?
                );

                found_count += 1;
                found_idxs.push(i);
            }
        }

        if required && (found_count == 0) {
            Err(Error::MissingPositional(format!("{}...", name)))
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
#[macro_use]
pub mod test_helpers {
    #[macro_export]
    macro_rules! string_vec {
        ( $($x:expr),* ) => {
            vec!( $(($x.to_string()),)* )
        }
    }
}

#[cfg(test)]
mod handle_args {
    use super::*;

    #[test]
    fn as_string_vec() {
        let mut verbosity = 0;
        let test_args = string_vec!("a", "b", "c");
        assert!(test_args.len() == 3);
        Parser::from_strings(test_args)
            .arg('v', "verbose", "increase verbosity with each given", &mut verbosity, None, false)
                .expect("failed to handle verbose argument(s)")
        ;
    }

    #[test]
    fn as_args_iter() {
        let mut verbosity: u64 = 0;
        Parser::from_args()
            .arg('v', "verbose", "increase verbosity with each given", &mut verbosity, None, false)
                .expect("failed to handle verbose argument(s)")
        ;
    }
}
