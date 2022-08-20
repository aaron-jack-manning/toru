use crate::colour;

use std::io;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Confy(confy::ConfyError),
    Trash(trash::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    Generic(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{} {}", colour::error("Internal Error:"), err),
            Error::Confy(err) => write!(f, "{} {}", colour::error("Internal Error:"), err),
            Error::Trash(err) => write!(f, "{} {}", colour::error("Internal Error:"), err),
            Error::TomlDe(err) => write!(f, "{} {}", colour::error("Internal Error:"), err),
            Error::TomlSer(err) => write!(f, "{} {}", colour::error("Internal Error:"), err),
            Error::Generic(message) => write!(f, "{}", message),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err : io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<confy::ConfyError> for Error {
    fn from(err : confy::ConfyError) -> Self {
        Error::Confy(err)
    }
}

impl From<trash::Error> for Error {
    fn from(err : trash::Error) -> Self {
        Error::Trash(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err : toml::de::Error) -> Self {
        Error::TomlDe(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err : toml::ser::Error) -> Self {
        Error::TomlSer(err)
    }
}
