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
            run_masks: BTreeMap::new(),
            walk_depth: 0,
            commit_depth: 0,
            max_depth: 0,
            parse_done: false,
            curr_group: None,

            help: false,
            has_variadic: false,
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
                for i in 0..arg.len() {
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

        if self.run_masks.contains_key(&idx) {
            return self.handle_run(idx, short, expect_value);
        }

        let arg = &self.args[idx];
        if arg.len() < 2 {
            return Ok(None);
        }

        let mut chars = arg.chars();
        let arg_0 = chars.next().or(Some('\0')).unwrap();

        // expect arg[0] to be '-'  -- otherwise, looks like a positional
        if arg_0 != '-' {
            return Ok(None);
        }

        let arg_1 = chars.next().or(Some('\0')).unwrap();
        let arg_2 = chars.next().or(Some('\0')).unwrap();

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

    // TODO: the help flags should be stored on `self` which is why this is
    // a member function. once the flag(s) are configurable we will store them
    // on the parser for this case
    fn is_help_flags(&self, short: char, long: &'static str) -> bool  {
        (short == 'h') || (long == "help")
    }
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

        if self.wants_help() {
            self.printer.add_subcommand(printer::Subcommand::new(name, desc));
            // do not return, subcommands need to continue parsing to set levels
            // and help appropriately
        }

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
        if self.wants_help() {
            self.printer.add_group(name, desc)?;
        }
        Ok(self)
    }


    //----------------------------------------------------------------
    // positional(s)
    //----------------------------------------------------------------

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
                return Err(Error::MissingPositional(name.to_string()));
            }
        };
        let val = &self.args[idx];
        *into = T::from_str(val)
            .map_err(|e| Error::PositionalConstructionError(name, format!("{}", e)))?;

        self.mask.remove(idx);

        Ok(self)
    }

    pub fn positional_list<'a, T: ToString + FromStr>(&'a mut self,
        name: &'static str, desc: &'static str,
        into: &mut Vec<T>, required: bool
    ) -> Result<&'a mut Parser, Error>
        where <T as FromStr>::Err: std::fmt::Display
    {
        if self.should_ignore(false) { return Ok(self); }

        if self.has_variadic {
            return Err(Error::MultipleVariadic(name));
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
