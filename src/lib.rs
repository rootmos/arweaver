use serde::{Deserialize};

extern crate reqwest;
use reqwest::Url;

pub struct Client {
    url: Url,
}

#[derive(Deserialize)]
pub struct Info {
    pub height: u64
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    UrlError(reqwest::UrlError),
    ReqwestError(reqwest::Error),
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

impl Client {
    pub fn new() -> Result<Client, Error> {
        let url = Url::parse(&std::env::var("ARWEAVE_TARGET")
                             .unwrap_or("https://arweave.net".to_string()))?;
        Ok(Client { url })
    }

    pub fn info(&self) -> Result<Info, Error> {
        Ok(reqwest::get(self.url.join("/info")?)?.json()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info() {
        let c = Client::new().unwrap();
        let i = c.info().unwrap();
        assert!(i.height > 316893);
    }
}
