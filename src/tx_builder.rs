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

    pub fn target(self, target: Address) -> Self {
        TxBuilder { target: Some(target), ..self }
    }

    pub fn data(self, data: Data) -> Self {
        TxBuilder { data, ..self }
    }

    pub fn quantity(self, quantity: Winstons) -> Self {
        TxBuilder { quantity, ..self }
    }

    pub fn reward(self, client: &Client) -> Result<Self, Error> {
        let reward = Some(client.price(self.target.as_ref(), self.data.len())?);
        Ok(TxBuilder { reward, ..self })
    }

    pub fn sign<W: AsRef<Wallet>>(self, wallet: W) -> Result<Tx, Error> {
        let txb = TxBuilder {
            owner: Some(wallet.as_ref().owner().clone()?),
            ..self
        };
        let mut s = Signer::new(wallet.as_ref().key())?;
        txb.squeeze(&mut s)?;
        let signature = Signature::new(s.sign()?)?;
        let id = signature.to_transaction_hash()?;
        let reward = txb.reward.ok_or(Error::value_not_present("reward", "request builder"))?;
        Ok(Tx {
            anchor: txb.anchor,
            data: txb.data,
            signature,
            id,
            owner: wallet.as_ref().owner().clone()?,
            quantity: txb.quantity,
            reward: reward,
            tags: txb.tags,
            target: EmptyStringAsNone::from(txb.target),
        })
    }
}
