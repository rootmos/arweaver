use reqwest::Url;
use arweave::{Error};

pub struct Client {
    url: Url,
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let url = Url::parse(&std::env::var("LOOM_TARGET")?)?;
        Ok(Client { url })
    }
}
