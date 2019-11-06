extern crate reqwest;
use reqwest::Url;

use crate::types::*;

pub struct Client {
    url: Url,
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
        Ok(reqwest::get(self.url.join("info")?)?.json()?)
    }

    pub fn block(&self, bh: &BlockHash) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/hash/")?.join(&bh.encode())?)?.json()?)
    }

    pub fn height(&self, h: Height) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/height/")?.join(&h.to_string())?)?.json()?)
    }

    pub fn current_block(&self) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/current")?)?.json()?)
    }
}
