#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UrlError(reqwest::UrlError),
    ReqwestError(reqwest::Error),
    Base64Error(base64::DecodeError),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::IoError(e) }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self { Error::ReqwestError(e) }
}

impl std::convert::From<reqwest::UrlError> for Error {
    fn from(e: reqwest::UrlError) -> Self { Error::UrlError(e) }
}

impl std::convert::From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self { Error::Base64Error(e) }
}
