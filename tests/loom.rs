use reqwest::Url;
use arweave::{Error, Address, Winstons, TxHash};
use serde::{Serialize, Deserialize};

pub struct Client {
    url: Url,
}

#[derive(Serialize, Debug)]
struct FaucetReq<'a> {
    beneficiary: &'a Address,
    quantity: &'a Winstons,
}

#[derive(Deserialize, Debug)]
struct FaucetRsp {
    tx_id: TxHash,
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let url = Url::parse(&std::env::var("LOOM_TARGET")?)?;
        Ok(Client { url })
    }

    pub fn faucet<A, Q>(&self, a: A, q: Q) -> Result<TxHash, Error>
    where A: AsRef<Address>, Q: AsRef<Winstons> {
        let client = reqwest::Client::new();

        let req = FaucetReq { beneficiary: a.as_ref(), quantity: q.as_ref() };
        let mut rsp = client.post(self.url.join("faucet")?).json(&req).send()?;
        Ok(rsp.json::<FaucetRsp>()?.tx_id)
    }
}
