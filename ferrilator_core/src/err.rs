#[derive(Debug, PartialEq)]
pub enum Error {
    Input(String),
    Io(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Input(msg) => write!(f, "input: {msg}"),
            Error::Io(msg) => write!(f, "io: {msg}"),
        }
    }
}

#[macro_export]
macro_rules! io {
    ($fmt:literal) => {
        Err(err::Error::Io(format!($fmt)))
    };
    ($fmt:literal, $($val:expr),*) => {
        Err(err::Error::Io(format!($fmt, $($val),*)))
    };
}
pub use io;

#[macro_export]
macro_rules! input {
    ($fmt:literal) => {
        Err(err::Error::Input(format!($fmt)))
    };
    ($fmt:literal, $($val:expr),*) => {
        Err(err::Error::Input(format!($fmt, $($val),*)))
    };
}
pub use input;

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Error {
        Error::Input(e.to_string())
    }
}

impl From<proc_macro2::LexError> for Error {
    fn from(e: proc_macro2::LexError) -> Error {
        Error::Input(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e.to_string())
    }
}
