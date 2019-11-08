use std::fmt;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UrlError(reqwest::UrlError),
    ReqwestError(reqwest::Error),
    OpensslError(openssl::error::ErrorStack),
    InvalidValue { thing: String, msg: String },
}

impl Error {
    pub fn invalid_value(thing: &str, msg: &str) -> Error {
        Error::InvalidValue { thing: thing.to_string(), msg: msg.to_string() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "io: {}", e),
            Error::ReqwestError(e) => write!(f, "request: {}", e),
            Error::UrlError(e) => write!(f, "url: {}", e),
            Error::OpensslError(e) => write!(f, "openssl: {}", e),
            Error::InvalidValue { thing, msg } => write!(f, "parsing {}: {}", thing, msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::IoError(e) }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self { Error::ReqwestError(e) }
}

impl From<reqwest::UrlError> for Error {
    fn from(e: reqwest::UrlError) -> Self { Error::UrlError(e) }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(e: openssl::error::ErrorStack) -> Self { Error::OpensslError(e) }
}
