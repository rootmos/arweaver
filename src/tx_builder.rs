use crate::types::*;
use crate::sponge::{Sponge, Absorbable, Signer};
use crate::error::Error;
use crate::client::Client;

pub struct TxBuilder {
    anchor: Anchor,
    owner: Option<Owner>,
    target: Option<Address>,
    data: Data,
    quantity: Winstons,
    reward: Option<Winstons>,
    tags: Tags,
}

impl Absorbable for TxBuilder {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        let owner = self.owner.as_ref().ok_or(Error::value_not_present("owner", "request builder"))?;
        let reward = self.reward.as_ref().ok_or(Error::value_not_present("reward", "request builder"))?;

        // https://github.com/ArweaveTeam/arweave/blob/d882d8a5880b765cd9a65928eaf7c04ea6aedfea/src/ar_tx.erl#L54
        owner.squeeze(s)?;
        if let Some(a) = self.target.as_ref() { a.squeeze(s)?; }
        self.data.squeeze(s)?;
        self.quantity.squeeze(s)?;
        reward.squeeze(s)?;
        self.anchor.squeeze(s)?;
        self.tags.squeeze(s)?;
        Ok(())
    }
}

impl TxBuilder {
    pub fn new(anchor: Anchor) -> Self {
        TxBuilder {
            anchor,
            owner: None,
            target: None,
            quantity: Winstons::from(0u32),
            reward: None,
            data: Data::from(vec![]),
            tags: Tags::new(),
        }
    }

    pub fn target(mut self, target: Address) -> Self {
        self.target = Some(target); self
    }

    pub fn data(mut self, data: Data) -> Self {
        self.data = data; self
    }

    pub fn quantity(mut self, quantity: Winstons) -> Self {
        self.quantity = quantity; self
    }

    pub fn reward(mut self, client: &Client) -> Result<Self, Error> {
        self.reward = Some(client.price(self.target.as_ref(), self.data.len())?);
        Ok(self)
    }

    pub fn sign<W: AsRef<Wallet>>(mut self, wallet: W) -> Result<Tx, Error> {
        self.owner = Some(wallet.as_ref().owner().clone()?);
        let mut s = Signer::new(wallet.as_ref().key())?;
        self.squeeze(&mut s)?;
        let signature = Signature::new(s.sign()?)?;
        let id = signature.to_transaction_hash()?;
        let reward = self.reward.ok_or(Error::value_not_present("reward", "request builder"))?;
        Ok(Tx {
            anchor: self.anchor,
            data: self.data,
            signature,
            id,
            owner: wallet.as_ref().owner().clone()?,
            quantity: self.quantity,
            reward: reward,
            tags: self.tags,
            target: EmptyStringAsNone::from(self.target),
        })
    }
}
