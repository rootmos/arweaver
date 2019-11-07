use std::fmt;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UrlError(reqwest::UrlError),
    ReqwestError(reqwest::Error),
    InvalidValue { thing: String, msg: String },
}

impl Error {
    pub fn invalid_value(thing: &str, msg: &str) -> Error {
        Error::InvalidValue { thing: thing.to_string(), msg: msg.to_string() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!();
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
