use reqwest::Url;

use crate::types::*;
use crate::error::*;

pub struct Client {
    url: Url,
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

    pub fn block<T: AsRef<BlockHash>>(&self, t: T) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/hash/")?.join(&t.as_ref().encode())?)?.json()?)
    }

    pub fn height<T: AsRef<Height>>(&self, t: T) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/height/")?.join(&t.as_ref().to_string())?)?.json()?)
    }

    pub fn current_block(&self) -> Result<Block, Error> {
        Ok(reqwest::get(self.url.join("block/current")?)?.json()?)
    }

    pub fn tx<T: AsRef<TxHash>>(&self, t: T) -> Result<Tx, Error> {
        Ok(reqwest::get(self.url.join("tx/")?.join(&t.as_ref().encode())?)?.json()?)
    }

    pub fn submit<T: AsRef<Tx>>(&self, t: T) -> Result<(), Error> {
        let client = reqwest::Client::new();
        client.post(self.url.join("tx")?).json(t.as_ref()).send()?;
        Ok(())
    }

    pub fn balance<T: AsRef<Address>>(&self, t: T) -> Result<Winstons, Error> {
        let url = self.url.join(&format!("wallet/{}/balance", t.as_ref().encode()))?;
        Ok(Winstons::decode(reqwest::get(url)?.text()?)?)
    }

    pub fn price<T: AsRef<Address>>(&self, t: Option<T>, size: usize) -> Result<Winstons, Error> {
        let url = match t {
            Some(target) => self.url.join(&format!("price/{}/{}", size, target.as_ref().encode()))?,
            None => self.url.join(&format!("price/{}", size))?,
        };
        Ok(Winstons::decode(reqwest::get(url)?.text()?)?)
    }
}
