use std::error::Error as ErrorImpl;

use crate::printer::arg_string;

pub enum Error {
    InvalidState(&'static str),
    InvalidInput(char, &'static str, &'static str),
    MissingArgValue(char, &'static str),
    ConstructionError(char, &'static str, String), // TODO: would be nice to keep the typed-error
    SubConstructionError(&'static str, String), // TODO: would be nice to keep the typed-error

    NestedGroup(&'static str, &'static str), // existing, attempted
    PrinterMissingGroup(&'static str),
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::InvalidState(_) => {
                "invalid parser state"
            }
            Error::InvalidInput(_, _, _) => {
                "invalid input"
            }
            Error::MissingArgValue(_, _) => {
                "missing argument value"
            }
            Error::ConstructionError(_, _, _) => {
                "failed to construct target from string"
            }
            Error::SubConstructionError(_, _) => {
                "failed to construct subcommand from string"
            }

            Error::NestedGroup(_, _) => {
                "groups cannot be nested"
            }
            Error::PrinterMissingGroup(_) => {
                "cannot add option to unknown group"
            }
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidState(desc) => {
                write!(f, "{}: {}", self.description(), desc)
            }
            Error::InvalidInput(short, long, desc) => {
                write!(f, "{}: {} {}", self.description(), arg_string(*short, long, false), desc)
            }
            Error::MissingArgValue(short, long) => {
                write!(f, "{} for {}", self.description(), arg_string(*short, long, false))
            }
            Error::ConstructionError(short, long, err) => {
                write!(f, "{} for {}: {}", self.description(), arg_string(*short, long, false), err)
            }
            Error::SubConstructionError(name, err) => {
                write!(f, "{} for {}: {}", self.description(), name, err)
            }


            Error::NestedGroup(orig, attempt) => {
                write!(f, "{} ({} within {})", self.description(), attempt, orig)
            }
            Error::PrinterMissingGroup(name) => {
                write!(f, "{} : {}", self.description(), name)
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
