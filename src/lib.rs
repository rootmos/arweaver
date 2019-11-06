use std::fmt;

use serde::{Deserialize, Deserializer};
use serde::de;

extern crate reqwest;
use reqwest::Url;

pub struct Client {
    url: Url,
}

#[derive(PartialEq, Eq)]
pub struct BlockHash(Vec<u8>);

impl BlockHash {
    pub fn encode(&self) -> String {
        base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockHash({})", self.encode())
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Height(u64);

impl std::convert::From<u64> for Height {
    fn from(n: u64) -> Self { Self(n) }
}

impl std::ops::Add for Height {
    type Output = Self;
    fn add(self, other: Self) -> Self { Self(self.0 + other.0) }
}

impl std::ops::Sub for Height {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        if self.0 < other.0 {
            Self(0)
        } else {
            Self(self.0 - other.0)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "indep_hash")]
    pub indep: BlockHash,
    pub previous_block: BlockHash,
    pub height: Height,
}

impl<'de> Deserialize<'de> for BlockHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BlockHashVisitor;
        impl<'de> de::Visitor<'de> for BlockHashVisitor {
            type Value = BlockHash;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("block hash")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let mut bh = BlockHash(vec![0; 48]);
                match base64::decode_config_slice(v, base64::URL_SAFE_NO_PAD, &mut bh.0) {
                    Ok(48) => Ok(bh),
                    _ => Err(de::Error::custom("should be 48 bytes base64 URL-safe encoded"))
                }
            }
        }

        deserializer.deserialize_str(BlockHashVisitor)
    }
}

#[derive(Deserialize)]
pub struct Info {
    pub height: Height,
    pub current: BlockHash,
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
        Ok(reqwest::get(self.url.join("block/height/")?.join(&h.0.to_string())?)?.json()?)
    }

    pub fn current_block(&self) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/current")?)?.json()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info() {
        let c = Client::new().unwrap();
        let i = c.info().unwrap();
        assert!(i.height > Height(316893));
    }

    #[test]
    fn block() {
        let c = Client::new().unwrap();
        let i = c.info().unwrap();
        let b0 = c.block(&i.current).unwrap();
        assert_eq!(b0.indep, i.current);
        assert_eq!(i.height, b0.height);

        let b1 = c.block(&b0.previous_block).unwrap();
        assert_eq!(b1.indep, b0.previous_block);
        assert_eq!(b1.height + Height::from(1), b0.height);
    }

    #[test]
    fn current_block() {
        let c = Client::new().unwrap();
        let b0 = c.current_block().unwrap();
        let b1 = c.block(&b0.indep).unwrap();
        assert_eq!(b0.indep, b1.indep);
        assert_eq!(b0.height, b1.height);
    }

    #[test]
    fn height() {
        let c = Client::new().unwrap();
        let i = c.info().unwrap();

        let b0 = c.height(i.height).unwrap();
        assert_eq!(b0.indep, i.current);
        assert_eq!(b0.height, i.height);

        let b1 = c.height(i.height - Height::from(1)).unwrap();
        assert_eq!(b1.indep, b0.previous_block);
        assert_eq!(b1.height + Height::from(1), b0.height);
    }
}
