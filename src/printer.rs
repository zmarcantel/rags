use std::collections::BTreeMap;

use crate::errors::Error;

const LEFT_PAD_LENGTH: usize = 4;
const MID_PAD_LENGTH: usize = 8;

pub fn arg_string(short: char, long: &'static str, prefix_long: bool) -> String {
    if short.is_alphabetic() && (long.len() > 0) {
        format!("-{}, --{}", short, long)
    } else if short.is_alphabetic() && long.is_empty() {
        format!("-{}", short)
    } else if !long.is_empty() {
        if prefix_long {
            format!("    --{}", long)
        } else {
            format!("--{}", long)
        }
    } else {
        // TODO: no panic -- should come up in testing
        unreachable!("unknown arg_string condiditon");
    }
}

// calculates the length of the result of `arg_string(...)` without
// allocating/creating the string itself
pub fn arg_string_len(short: char, long: &'static str) -> usize {
    let len_short = 2; /* '-c' */
    let len_sep = 2; /* ', ' */
    let len_long = 2 + long.len(); /* '--long' */
    if short.is_alphabetic() && (long.len() > 0) {
        len_short + len_sep + len_long
    } else if short.is_alphabetic() && long.is_empty() {
        len_short
    } else if !long.is_empty() {
        // same as a fully-specified arg because we pad the short and separator
        len_short + len_sep + len_long
    } else {
        // TODO: no panic -- should come up in testing
        unreachable!("unknown arg_string_len condiditon");
    }
}

trait Descriptor {
    fn left_len(&self) -> usize;
}
trait Printable {
    fn should_print(&self) -> bool;
    fn print(&self, left_pad: usize, longest_left: usize);
}


pub struct Argument {
    short: char,
    long: &'static str,
    desc: &'static str,
    label: Option<&'static str>,
    default: Option<String>,
    required: bool,
}
impl Argument {
    // TODO: take item name and default value
    pub fn new(
        short: char, long: &'static str, desc: &'static str,
        label: Option<&'static str>, default: Option<String>,
        required: bool
    ) -> Argument
    {
        Argument{
            short: short,
            long: long,
            desc: desc,
            label: label,
            default: default,
            required: required,
        }
    }

    pub fn arg_string(&self) -> String {
        if let Some(l) = self.label {
            format!("{} {}", arg_string(self.short, self.long, true), l)
        } else {
            arg_string(self.short, self.long, true)
        }
    }
}
impl Descriptor for Argument {
    fn left_len(&self) -> usize {
        let base = arg_string_len(self.short, self.long);
        if let Some(l) = self.label {
            base + 1 + l.len()
        } else {
            base
        }
    }
}
impl Printable for Argument {
    fn should_print(&self) -> bool {
        self.short.is_alphabetic() || (!self.long.is_empty())
    }
    fn print(&self, left_pad: usize, longest_left: usize) {
        let args = self.arg_string();
        let left = " ".repeat(left_pad);
        let mid = " ".repeat(longest_left - args.len() + MID_PAD_LENGTH);

        let has_default = self.default.is_some();
        let accesories = if has_default && self.required {
            format!(" [required, default: {}]", self.default.as_ref().unwrap())
        } else if has_default {
            format!(" [default: {}]", self.default.as_ref().unwrap())
        } else if self.required {
            " [required]".to_string()
        } else {
            "".to_string()
        };

        println!("{}{}{}{}{}", left, args, mid, self.desc, accesories);
    }
}

pub struct Subcommand {
    name: &'static str,
    desc: &'static str,
}
impl Subcommand {
    pub fn new(name: &'static str, desc: &'static str) -> Subcommand {
        Subcommand{
            name: name,
            desc: desc,
        }
    }
}
impl Descriptor for Subcommand {
    fn left_len(&self) -> usize {
        self.name.len()
    }
}
impl Printable for Subcommand {
    fn should_print(&self) -> bool {
        !self.name.is_empty()
    }
    fn print(&self, left_pad: usize, longest_left: usize) {
        let left = " ".repeat(left_pad);
        let mid = " ".repeat(longest_left - self.name.len() + MID_PAD_LENGTH);
        println!("{}{}{}{}", left, self.name, mid, self.desc);
    }
}

pub struct Group {
    name:&'static str,
    desc:&'static str,
    opts: Vec<Argument>,
}
impl Group {
    pub fn new(name: &'static str, desc: &'static str) -> Group {
        Group {
            name: name,
            desc: desc,
            opts: vec!(),
        }
    }
}
impl Printable for Group {
    fn should_print(&self) -> bool {
        (!self.name.is_empty()) && (!self.opts.is_empty())
    }
    fn print(&self, left_pad: usize, longest_left: usize) {
        let mid = " ".repeat(
            // get the basic padding based on the naem
            longest_left - self.name.len() +
            // we do not pad left, so add that back in
            // add in the middle padding all args share
            // subtract the ':' after the name
            LEFT_PAD_LENGTH + MID_PAD_LENGTH - 1
        );
        println!("{}:{}{}", self.name, mid, self.desc);
        for o in self.opts.iter() {
            o.print(left_pad + LEFT_PAD_LENGTH, longest_left);
        }
    }
}

pub struct App {
    name: &'static str,
    short_desc: &'static str,
    long_desc: &'static str,
    version: &'static str,
}
impl App {
    pub fn empty() -> App {
        App::new("", "", "", "")
    }

    pub fn new(
        name: &'static str, short: &'static str, long: &'static str,
        vers: &'static str
    ) -> App {
        App{
            name: name,
            short_desc: short,
            long_desc: long,
            version: vers,
        }
    }
}
impl Printable for App {
    fn should_print(&self) -> bool {
        self.name.is_empty() == false
    }
    fn print(&self, _: usize, _: usize) {
        let has_name = !self.name.is_empty();
        let has_vers = !self.version.is_empty();
        let has_desc = !self.short_desc.is_empty();

        if has_name && has_vers && has_desc {
            println!("{} {} - {}", self.name, self.version, self.short_desc);
        } else if has_name && has_vers {
            println!("{} {}", self.name, self.version);
        } else if has_name {
            println!("{}", self.name);
        }
        println!("");
    }
}


pub struct Printer {
    app: App,
    subs: Vec<Subcommand>,
    groups: BTreeMap<&'static str, Group>,
    opts: Vec<Argument>,

    longest_left: usize,
}
impl Printer {
    pub fn new(app: App) -> Printer {
        Printer {
            app: app,
            subs: vec!(),
            groups: BTreeMap::new(),
            opts: vec!(),

            longest_left: 0usize,
        }
    }

    pub fn new_level(&mut self) {
        self.subs.clear();
    }

    pub fn print(&self) {
        if self.app.should_print() {
            self.app.print(0, 0);
        }

        let group_args_count = self.groups.iter()
            .fold(0, |acc, (_, grp)| acc + grp.opts.len());
        let has_args = (!self.opts.is_empty()) || (group_args_count > 0);

        if has_args {
            println!("usage: {} {}", self.app.name, self.generate_usage());
            println!("");
        }

        if !self.app.long_desc.is_empty() {
            println!("{}", self.app.long_desc);
            println!("");
        }

        if !self.subs.is_empty() {
            println!("subcommands:");
            for s in self.subs.iter() {
                if !s.should_print() { continue; }
                s.print(LEFT_PAD_LENGTH, self.longest_left);
            }
            println!("");
        }

        for (_, desc) in self.groups.iter() {
            if !desc.should_print() { continue; }
            desc.print(0, self.longest_left); // NOTE: groups print at left-offset 0
            println!("");
        }

        if !self.opts.is_empty() {
            println!("options:");
            for o in self.opts.iter() {
                if !o.should_print() { continue; }
                o.print(LEFT_PAD_LENGTH, self.longest_left);
            }
        }

        println!("");
    }

    fn calculate_longest<T: Descriptor>(&mut self, desc: &T) {
        self.longest_left = std::cmp::max(self.longest_left, desc.left_len());
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.app.name = name;
    }
    pub fn set_version(&mut self, vers: &'static str) {
        self.app.version = vers;
    }
    pub fn set_short_desc(&mut self, desc: &'static str) {
        self.app.short_desc = desc;
    }
    pub fn set_long_desc(&mut self, desc: &'static str) {
        self.app.long_desc = desc;
    }

    fn generate_usage(&self) -> String {
        let mut opt_shorts: Vec<String> = vec!();
        let mut opt_longs: Vec<String> = vec!();
        let mut req_shorts: Vec<String> = vec!();
        let mut req_longs: Vec<String> = vec!();

        let mut filter_opt = |o: &Argument| {
            let (result, is_long) = if o.short.is_alphabetic() {
                if let Some(label) = o.label {
                    (format!("-{} {}", o.short, label), true)
                } else {
                    (o.short.to_string(), false)
                }
            } else {
                if let Some(label) = o.label {
                    (format!("--{} {}", o.long, label), true)
                } else {
                    (format!("--{}", o.long), true)
                }
            };

            if o.required {
                if is_long { req_longs.push(result); }
                else { req_shorts.push(result); }
            } else {
                if is_long { opt_longs.push(result); }
                else { opt_shorts.push(result); }
            }
        };

        for o in self.opts.iter() {
            filter_opt(o);
        }

        for (_, g) in self.groups.iter() {
            for o in g.opts.iter() {
                filter_opt(o);
            }
        }

        let opt_short_string = opt_shorts.join("");
        let opt_long_string = opt_longs.join(" ");

        let req_short_string = req_shorts.join("");
        let req_long_string = req_longs.join(" ");

        let opts = if (!opt_short_string.is_empty()) && (!opt_long_string.is_empty()) {
            format!("[-{} {}]", opt_short_string, opt_long_string)
        } else if !opt_short_string.is_empty() {
            format!("[-{}]", opt_short_string)
        } else if !opt_long_string.is_empty() {
            format!("[{}]", opt_long_string)
        } else {
            "".to_string()
        };

        let reqs = if (!req_short_string.is_empty()) && (!req_long_string.is_empty()) {
            format!("-{} {}", req_short_string, req_long_string)
        } else if !req_short_string.is_empty() {
            format!("-{}", req_short_string)
        } else if !req_long_string.is_empty() {
            format!("{}", req_long_string)
        } else {
            "".to_string()
        };

        let arg_usage = if (!opts.is_empty()) && (!reqs.is_empty()) {
            format!("{} {}", opts, reqs)
        } else if !opts.is_empty() {
            opts
        } else if !opts.is_empty() {
            reqs
        } else {
            "".to_string()
        };

        if !self.subs.is_empty() {
            format!("{{subcommand}} {}", arg_usage)
        } else {
            arg_usage
        }
    }


    pub fn add_subcommand(&mut self, sub: Subcommand) {
        // TODO: sanity checking?
        self.calculate_longest(&sub);
        self.subs.push(sub);
    }
    pub fn add_group(&mut self, name: &'static str, desc: &'static str) -> Result<(), Error> {
        self.groups.insert(name, Group::new(name, desc));
        Ok(())
    }
    pub fn add_arg(&mut self, opt: Argument, grp: Option<&'static str>) -> Result<(), Error> {
        // TODO: sanity checking?
        self.calculate_longest(&opt);

        if grp.is_none() {
            self.opts.push(opt);
            return Ok(());
        }

        let grpname = grp.unwrap();
        match self.groups.get_mut(grpname) {
            Some(g) => {
                g.opts.push(opt);
                Ok(())
            }
            None => {
                Err(Error::PrinterMissingGroup(grpname))
            }
        }
    }
}
