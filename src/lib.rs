use std::env;
use std::str::FromStr;
use std::string::ToString;

extern crate bit_set;

pub mod errors;
pub use errors::*;

mod printer;
use printer::arg_string;

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


#[derive(Debug)]
enum ValueLocation {
    Unknown,
    HasEqual(usize),
    TakesNext,
}

struct FoundMatch {
    index: usize,
    loc: ValueLocation,
}
impl FoundMatch {
    pub fn new(idx: usize, loc: ValueLocation) -> FoundMatch {
        FoundMatch {
            index: idx,
            loc: loc,
        }
    }
}



pub struct Parser {
    args: Vec<String>,
    mask: bit_set::BitSet,

    walk_depth: usize,
    commit_depth: usize,
    max_depth: usize,
    parse_done: bool,
    curr_group: Option<&'static str>,

    help: bool,
    printer: printer::Printer,
}
impl Parser {
    pub fn from_strings(input: Vec<String>) -> Parser {
        let count = input.len();
        let mut bits = bit_set::BitSet::with_capacity(count);
        // TODO: PR with BitSet::set_all() -- or an inverse iter that iterates all unset
        for i in 1..count {
            bits.insert(i);
        }

        let mut p = Parser{
            args: input,
            mask: bits,
            walk_depth: 0,
            commit_depth: 0,
            max_depth: 0,
            parse_done: false,
            curr_group: None,

            help: true, //force it on so the -h/--help gets added to help. will disable itself
            printer: printer::Printer::new(printer::App::empty()),
        };

        let mut wants_help = false;
        p.flag('h', "help", "print this help dialog", &mut wants_help, false)
            .expect("could not handle help flag");
        p.help = wants_help;

        p
    }
    pub fn from_args() -> Parser {
        let args = env::args().collect::<Vec<String>>();
        Parser::from_strings(args)
    }


    //----------------------------------------------------------------
    // help setup
    //----------------------------------------------------------------

    pub fn app_name<'a>(&'a mut self, name: &'static str) -> &'a mut Parser {
        self.printer.set_name(name);
        self
    }

    pub fn app_desc<'a>(&'a mut self, desc: &'static str) -> &'a mut Parser {
        self.printer.set_short_desc(desc);
        self
    }

    pub fn app_long_desc<'a>(&'a mut self, desc: &'static str) -> &'a mut Parser {
        self.printer.set_long_desc(desc);
        self
    }

    pub fn app_version<'a>(&'a mut self, vers: &'static str) -> &'a mut Parser {
        self.printer.set_version(vers);
        self
    }


    pub fn wants_help(&self) -> bool {
        self.help
    }

    pub fn print_help(&self) {
        self.printer.print();
    }


    //----------------------------------------------------------------
    // parse helpers
    //----------------------------------------------------------------

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


    // returns (matches, info)
    fn matches_short(&self, idx: usize, short: char, expect_value: bool) -> (bool, ValueLocation) {
        if short == '\0' { return (false, ValueLocation::Unknown); }

        let arg = &self.args[idx];
        if arg.len() < 2 {
            return (false, ValueLocation::Unknown);
        }

        let mut chars = arg.chars();
        let arg_0 = chars.next().or(Some('\0')).unwrap();
        let arg_1 = chars.next().or(Some('\0')).unwrap();
        let arg_2 = chars.next().or(Some('\0')).unwrap();

        // expect arg[0] to be '-'  -- otherwise, looks like a positional
        if arg_0 != '-' {
            return (false, ValueLocation::Unknown);
        }

        // expect arg[1] to be the character we are looking for
        if arg_1 != short {
            return (false, ValueLocation::Unknown);
        }

        // if we got here, and the length is 2, we have the base case so just return
        if arg.len() == 2 {
            let has_next = self.mask.contains(idx + 1);
            return (true, if expect_value && has_next {ValueLocation::TakesNext} else {ValueLocation::Unknown});
        }

        // if the arg has >2 characters, and arg[2] == '=', then we match and return the '=' offset
        if arg_2 == '=' {
            // return HasEqual regardless of expect_value because errors should be handled there
            // rather than this lower context
            return (true, ValueLocation::HasEqual(2));
        }

        // otherwise... no match
        (false, ValueLocation::Unknown)
    }

    fn matches_long(&self, idx: usize, long: &'static str, expect_value: bool) -> (bool, ValueLocation) {
        if long.is_empty() { return (false, ValueLocation::Unknown); }

        let arg = self.args[idx].as_str();
        let end_of_arg = 2 + long.len();

        // not enough string to match
        if arg.len() < end_of_arg {
            return (false, ValueLocation::Unknown);
        }

        // not a long arg
        if &arg[..2] != "--" {
            return (false, ValueLocation::Unknown);
        }

        if &arg[2..end_of_arg] != long {
            return (false, ValueLocation::Unknown);
        }

        // we got exactly what we were looking for, so return
        if arg.len() == end_of_arg {
            let has_next = self.mask.contains(idx + 1);
            return (
                true,
                if expect_value && has_next {
                    ValueLocation::TakesNext
                } else {
                    ValueLocation::Unknown
                }
            );
        }

        // we got here, so the string is longer than we expect
        // so check for a '=' trailing and return as such
        if let Some(c) = arg.chars().nth(end_of_arg) {
            if c == '=' {
                // return HasEqual regardless of expect_value because errors should be handled there
                // rather than this lower context
                return (true, ValueLocation::HasEqual(end_of_arg));
            }
        }

        // otherwise, no match
        (false, ValueLocation::Unknown)
    }

    fn find_match(&self, short: char, long: &'static str, expect_value: bool)
        -> Option<FoundMatch>
    {
        for i in self.mask.iter() {
            let (matches, arg_loc) = self.matches_short(i, short, expect_value);
            if matches {
                return Some(FoundMatch::new(i, arg_loc));
            }

            let (matches, arg_loc) = self.matches_long(i, long, expect_value);
            if matches {
                return Some(FoundMatch::new(i, arg_loc));
            }
        }

        None
    }

    fn find_subcommand(&self, name: &'static str) -> Option<FoundMatch> {
        for i in self.mask.iter() {
            let arg = &self.args[i];
            if arg == name {
                return Some(FoundMatch {
                    index: i,
                    loc: ValueLocation::Unknown,
                })
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
        match info.loc {
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
        }

        let found_opt = self.find_match(short, long, true);
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

    pub fn short_arg<'a, T: FromStr+ToString>(&'a mut self,
        short: char, desc: &'static str, into: &mut T, label: Option<&'static str>,
        required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        self.arg(short, "", desc, into, label, required)
    }

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
        }

        let found_opt = self.find_match(short, long, false);
        if found_opt.is_none() { // TODO: required flag
            return Ok(self);
        }

        let found = found_opt.unwrap();
        self.mask.remove(found.index);

        match found.loc {
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

    pub fn short_flag<'a>(&'a mut self,
        short: char, desc: &'static str,
        into: &mut bool, invert: bool
    ) -> Result<&'a mut Parser, Error>
    {
        self.flag(short, "", desc, into, invert)
    }

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
        }

        loop { // loop until we get no results back
            let found_opt = self.find_match(short, long, false);
            if found_opt.is_none() { // TODO: required count -- does this make sense?
                return Ok(self);
            }

            let found = found_opt.unwrap();
            self.mask.remove(found.index);

            match found.loc {
                ValueLocation::Unknown => {
                    into.add_assign(step.clone());
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

    pub fn short_count<'a, T: std::ops::AddAssign + ToString + Clone>(&'a mut self,
        short: char, desc: &'static str,
        into: &mut T, step: T
    ) -> Result<&'a mut Parser, Error>
    {
        self.count(short, "", desc, into, step)
    }

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
        }

        let mut found_count = 0;
        loop { // loop until we get no results back
            let found_opt = self.find_match(short, long, true);
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

            let ctor_result = match found.loc {
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

    pub fn short_list<'a, T: FromStr + ToString>(&'a mut self,
        short: char, desc: &'static str,
        into: &mut Vec<T>, label: Option<&'static str>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        self.list(short, "", desc, into, label, required)
    }

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

        // TODO: if self.wants_help()
        self.printer.add_subcommand(printer::Subcommand::new(name, desc));

        if name.is_empty() {
            return Err(Error::InvalidState("subcommand(...) given empty name"));
        }

        if let Some(info) = self.find_subcommand(name) {
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

    pub fn group<'a>(&'a mut self, name: &'static str, desc: &'static str)
        -> Result<&'a mut Parser, Error>
    {
        if let Some(orig) = self.curr_group {
            return Err(Error::NestedGroup(orig, name));
        }

        if self.should_ignore(false) { return Ok(self); }

        self.curr_group = Some(name);
        self.printer.add_group(name, desc)?;
        Ok(self)
    }

}

#[cfg(test)]
#[macro_use]
mod test_helpers {
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
            .arg('v', "verbose", "increase verbosity with each given", &mut verbosity, None)
                .expect("failed to handle verbose argument(s)")
        ;
    }

    #[test]
    fn as_args_iter() {
        let mut verbosity: u64 = 0;
        Parser::from_args()
            .arg('v', "verbose", "increase verbosity with each given", &mut verbosity, None)
                .expect("failed to handle verbose argument(s)")
        ;
    }
}

#[cfg(test)]
mod args {
    use super::*;

    #[test]
    fn basic() {
        let mut file: String = "default.file".to_string();
        let mut short: String = "".to_string();
        let mut long: String = "".to_string();

        let args = string_vec!("argv[0]", "-f", "foo.bar", "-s", "foo", "--long", "bar");
        Parser::from_strings(args)
            .arg('f', "file", "file to handle", &mut file, None)
                .expect("failed to parse file argument")
            .short_arg('s', "a short arg", &mut short, None)
                .expect("failed to parse short arg")
            .long_arg("long", "a long arg", &mut long, None)
                .expect("failed to parse short arg")
        ;

        assert!(file == "foo.bar", "got unexpected 'file' value: {}", file);
        assert!(short == "foo", "got unexpected 'short' value: {}", short);
        assert!(long == "bar", "got unexpected 'long' value: {}", long);
    }

    #[test]
    fn with_eq() {
        let mut file: String = "default.file".to_string();
        let mut short: u16 = 50;
        let mut long: usize = 100;

        let args = string_vec!("argv[0]", "-f=foo.bar", "-s=17", "--long", "10");
        Parser::from_strings(args)
            .arg('f', "file", "file to handle", &mut file, None)
                .expect("failed to parse file argument")
            .short_arg('s', "a short arg", &mut short, None)
                .expect("failed to parse short arg")
            .long_arg("long", "a long arg", &mut long, None)
                .expect("failed to parse short arg")
        ;

        assert!(file == "foo.bar", "got unexpected 'file' value: {}", file);
        assert!(short == 17, "got unexpected 'short' value: {}", short);
        assert!(long == 10, "got unexpected 'long' value: {}", long);
    }
}

#[cfg(test)]
mod flags {
    use super::*;

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


#[cfg(test)]
mod count {
    use super::*;

    // TODO: short_count(...) and long_count(...) tests

    fn generic_count<T>(initial: T, step: T, expect: T)
        where T: std::ops::AddAssign + Clone + std::fmt::Display + std::cmp::PartialEq
    {
        let mut count: T = initial;

        Parser::from_strings(string_vec!("argv[0]", "-v", "-v", "--verbose", "-v", "--verbose"))
            .count('v', "verbose", "increase verbosity", &mut count, step).expect("bad count parse")
        ;

        assert!(count == expect, "unexpected count value {}, wanted {}", count, expect);
    }

    #[test] fn u8() { generic_count::<u8>(0u8, 1u8, 5u8); }
    #[test] fn u8_stepped() { generic_count::<u8>(0u8, 10u8, 50); }

    #[test] fn u16() { generic_count::<u16>(0u16, 1u16, 5u16); }
    #[test] fn u16_stepped() { generic_count::<u16>(0u16, 8u16, 40u16); }

    #[test] fn u32() { generic_count::<u32>(0u32, 1u32, 5u32); }
    #[test] fn u32_stepped() { generic_count::<u32>(0u32, 2u32, 10u32); }

    #[test] fn u64() { generic_count::<u64>(0u64, 1u64, 5u64); }
    #[test] fn u64_stepped() { generic_count::<u64>(0u64, 3u64, 15u64); }

    #[test] fn usize() { generic_count::<usize>(0usize, 1usize, 5usize); }
    #[test] fn usize_stepped() { generic_count::<usize>(0usize, 6usize, 30usize); }


    #[test] fn i8() { generic_count::<i8>(0i8, 1i8, 5i8); }
    #[test] fn i8_stepped() { generic_count::<i8>(0i8, 4i8, 20i8); }

    #[test] fn i16() { generic_count::<i16>(0i16, 1i16, 5i16); }
    #[test] fn i16_stepped() { generic_count::<i16>(0i16, 8i16, 40i16); }

    #[test] fn i32() { generic_count::<i32>(0i32, 1i32, 5i32); }
    #[test] fn i32_stepped() { generic_count::<i32>(0i32, 2i32, 10i32); }

    #[test] fn i64() { generic_count::<i64>(0i64, 1i64, 5i64); }
    #[test] fn i64_stepped() { generic_count::<i64>(0i64, 9i64, 45i64); }

    #[test] fn isize() { generic_count::<isize>(0isize, 1isize, 5isize); }
    #[test] fn isize_stepped() { generic_count::<isize>(0isize, 10isize, 50isize); }


    #[test] fn f32() { generic_count::<f32>(0f32, 1f32, 5f32); }
    #[test] fn f32_stepped() { generic_count::<f32>(0f32, 1.5f32, 7.5f32); }

    #[test] fn f64() { generic_count::<f64>(0f64, 1f64, 5f64); }
    #[test] fn f64_stepped() { generic_count::<f64>(0f64, 3.2f64, 16f64); }
}


#[cfg(test)]
mod lists {
    use super::*;

    #[test]
    fn basic() {
        let mut test_list: Vec<String> = vec!();

        Parser::from_strings(string_vec!("argv[0]", "-f", "foo.bar", "--file", "bar.baz", "-f", "last"))
            .list('f', "file", "add file to list", &mut test_list, None).expect("bad list")
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
            .list('f', "file", "add file to list", &mut test_list, None).expect("bad list")
        ;

        assert!(test_list.len() == 3, "incorrect vector len {}", test_list.len());
        assert!(test_list[0] == "foo.bar", "incorrect vector value[0] {}", test_list[0]);
        assert!(test_list[1] == "bar.baz", "incorrect vector value[1] {}", test_list[1]);
        assert!(test_list[2] == "last", "incorrect vector value[2] {}", test_list[2]);
    }
}


#[cfg(test)]
mod subcommands {
    use super::*;

    #[test]
    fn basic() {
        let mut subs: Vec<String> = vec!();

        Parser::from_strings(string_vec!("argv[0]", "run", "until", "midnight", "-vvvvv"))
            .subcommand("build", "do a build", &mut subs).expect("bad sub(build)")
                .done().expect("no done on build")
            .subcommand("run", "run a target", &mut subs).expect("bad sub(run)")
                .subcommand("until", "run a target until a time", &mut subs).expect("bad sub-sub(until)")
                    .subcommand("midnight", "alias for passing in midnnight", &mut subs)
                        .expect("bad sub-sub-sub(midnight)")
                        .done().expect("no done on run-until-midnight")
                    .done().expect("no done on run-until")
                .done().expect("no done on run")
            .subcommand("test", "test a target", &mut subs).expect("bad sub(test)")
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
            .subcommand("build", "do a build", &mut subs)
                .expect("bad sub(build)")
                .arg('f', "file", "file to build", &mut build_file, None)
                    .expect("bad build-file")
                .done().expect("no done on build")
            .subcommand("test", "test a target", &mut subs).expect("bad sub(test)")
                .arg('f', "file", "file to test", &mut test_file, None)
                    .expect("bad test-file")
                .done().expect("no done on test")
        ;

        assert!(subs.len() == 1, "wrong sub count: {}", subs.len());
        assert!(subs[0] == "build", "did not take build path: {}", subs[0]);

        assert!(build_file == "hahaha.txt", "did not set build-file: {}", build_file);
        assert!(test_file == "test", "overwrote test-file: {}", test_file);
    }
}
