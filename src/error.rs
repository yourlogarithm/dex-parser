use std::{
    error,
    fmt::{self, Display},
    io,
};

#[derive(Debug)]
pub enum Error {
    MalFormed(String),
    IO(io::Error),
    InvalidId(String),
    Scroll(scroll::Error),
    BadOffset(usize, String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(_) => "IO error",
            Error::MalFormed(_) => "Entity is malformed in some way",
            Error::Scroll(_) => "Scroll error",
            Error::InvalidId(_) => "Invalid index",
            Error::BadOffset(_, _) => "Invalid offset",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IO(ref io) => io.source(),
            Error::Scroll(ref err) => err.source(),
            Error::MalFormed(_) => None,
            Error::InvalidId(_) => None,
            Error::BadOffset(_, _) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<scroll::Error> for Error {
    fn from(err: scroll::Error) -> Error {
        Error::Scroll(err)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IO(ref err) => write!(fmt, "{}", err),
            Error::Scroll(ref err) => write!(fmt, "{}", err),
            Error::MalFormed(ref msg) => write!(fmt, "Malformed entity: {}", msg),
            Error::InvalidId(ref msg) => write!(fmt, "{}", msg),
            Error::BadOffset(offset, ref msg) => write!(fmt, "{}: {}", msg, offset),
        }
    }
}
