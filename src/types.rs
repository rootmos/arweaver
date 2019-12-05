use std::fmt;
use std::convert::From;
use std::marker::PhantomData;

use crate::error::Error;
use crate::sponge::{Sponge, Absorbable, Verifier};

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use openssl::bn::BigNum;
use openssl::hash::{MessageDigest, hash};
use openssl::rsa::{Rsa};
use openssl::pkey::{PKey, PKeyRef, Public, Private, HasPublic};
use serde::de;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{IntoDeserializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyStringAsNone<T>(Option<T>);

impl<T> EmptyStringAsNone<T> {
    fn as_option_ref(&self) -> Option<&T> { self.0.as_ref() }
}

impl<T: Serialize> Serialize for EmptyStringAsNone<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.as_option_ref() {
            Some(t) => t.serialize(s),
            None => s.serialize_str(""),
        }
    }
}

impl<T> From<Option<T>> for EmptyStringAsNone<T> {
    fn from(ot: Option<T>) -> Self { Self(ot) }
}

struct EmptyStringAsNoneVisitor<T> {
    marker: PhantomData<T>
}

impl<'de, T> de::Visitor<'de> for EmptyStringAsNoneVisitor<T>
where
    T: Deserialize<'de>
{
    type Value = EmptyStringAsNone<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("empty string as absent value")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        if s.len() == 0 {
            Ok(EmptyStringAsNone(None))
        } else {
            T::deserialize(s.into_deserializer()).map(Some).map(EmptyStringAsNone)
        }
    }
}

impl<'de, T> Deserialize<'de> for EmptyStringAsNone<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(EmptyStringAsNoneVisitor { marker: PhantomData })
    }
}


#[derive(Hash, PartialEq, Eq, Clone)]
struct Bytes {
    bytes: Vec<u8>,
    thing: &'static str,
}

impl Bytes {
    #[inline] fn new<T: AsRef<[u8]>>(thing: &'static str, t: T) -> Self {
        Self { bytes: t.as_ref().to_owned(), thing }
    }

    #[inline] fn len(&self) -> usize { self.bytes.len() }
    #[inline] fn as_slice(&self) -> &[u8] { self.bytes.as_slice() }

    fn encode(&self) -> String {
        base64::encode_config(self.as_slice(), base64::URL_SAFE_NO_PAD)
    }

    fn decode<T: AsRef<[u8]>>(thing: &'static str, t: T) -> Result<Bytes, Error> {
        base64::decode_config(&t, base64::URL_SAFE_NO_PAD)
            .map(|bytes| Bytes { thing, bytes }).map_err(|_| {
                Error::invalid_value(thing, "invalid format (base64 URL-safe w/o padding)")
            })
    }

    fn with_expected_length(self, l: usize) -> Result<Bytes, Error> {
        if self.bytes.len() != l {
            Err(Error::invalid_value(
                    self.thing,
                    &format!("invalid length (is {}, should be {})", self.bytes.len(), l)))
        } else {
            Ok(self)
        }
    }
}

fn is_human_readable(s: &String) -> bool {
    s.chars().all(|c| {
        c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace()
    })
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match String::from_utf8(self.bytes.to_owned()) {
            Ok(ref s) if is_human_readable(&s) => write!(f, "{}", s),
            _ => write!(f, "{}", self.encode()),
        }
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;
    fn into_iter(self) -> std::slice::Iter<'a, u8> { self.bytes.iter() }
}

struct BytesVisitor {
    thing: &'static str,
    expected_length: Option<usize>,
}

impl BytesVisitor {
    fn new(thing: &'static str) -> BytesVisitor {
        BytesVisitor { thing, expected_length: None }
    }

    fn new_with_expected_length(thing: &'static str, expected_length: usize) -> BytesVisitor {
        BytesVisitor { thing, expected_length: Some(expected_length) }
    }
}

impl<'de> de::Visitor<'de> for BytesVisitor {
    type Value = Bytes;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.thing)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let mut r = Bytes::decode(self.thing, v);
        if let Some(l) = self.expected_length {
            r = r.and_then(|bs| bs.with_expected_length(l))
        }
        r.map_err(de::Error::custom)
    }
}

impl Serialize for Bytes {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.encode())
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct BlockHash(Bytes);

impl BlockHash {
    pub fn encode(&self) -> String {
        self.0.encode()
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        Bytes::decode("block hash", t).and_then(|bs| bs.with_expected_length(48)).map(Self)
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl AsRef<BlockHash> for BlockHash {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl Absorbable for BlockHash {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        s.absorb(&self.0.as_slice())
    }
}

impl<'de> Deserialize<'de> for BlockHash {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new_with_expected_length("block hash", 48)).map(Self)
    }
}


#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Height(u64);

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Height {
    #[inline] fn from(n: u64) -> Self { Self(n) }
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

impl AsRef<Height> for Height {
    #[inline] fn as_ref(&self) -> &Self { self }
}


#[derive(Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "indep_hash")]
    pub indep: BlockHash,
    previous_block: EmptyStringAsNone<BlockHash>,
    pub height: Height,
    pub txs: Vec<TxHash>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

impl Block {
    pub fn previous_block(&self) -> Option<&BlockHash> {
        self.previous_block.as_option_ref()
    }
}


#[derive(Deserialize)]
pub struct Info {
    pub height: Height,
    pub current: BlockHash,
}


#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct TxHash(Bytes);

impl TxHash {
    pub fn encode(&self) -> String {
        self.0.encode()
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        Bytes::decode("transaction hash", t).and_then(|bs| bs.with_expected_length(32)).map(Self)
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl AsRef<TxHash> for TxHash {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl Absorbable for TxHash {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        s.absorb(&self.0.as_slice())
    }
}

impl<'de> Deserialize<'de> for TxHash {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new_with_expected_length("transaction hash", 32)).map(Self)
    }
}


#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Data(Bytes);

impl Data {
    pub fn len(&self) -> usize { self.0.len() }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        Bytes::decode("data", t).map(Self)
    }
}

impl AsRef<Data> for Data {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl From<Vec<u8>> for Data {
    fn from(bytes: Vec<u8>) -> Data { Data(Bytes { thing: "data", bytes }) }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("data")).map(Self)
    }
}

impl Absorbable for Data {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        s.absorb(&self.0.as_slice())
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Winstons(BigUint);

impl Winstons {
    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        BigUint::parse_bytes(t.as_ref(), 10).map(Self).ok_or(
            Error::invalid_value("a non-negative decimal number of Winstons", "invalid format"))
    }
}

impl fmt::Display for Winstons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for Winstons {
    type Output = Winstons;
    fn add(self, other: Self) -> Self { Self(self.0 + other.0) }
}

impl std::ops::Add for &Winstons {
    type Output = Winstons;
    fn add(self, other: Self) -> Winstons { Winstons(self.0.to_owned() + other.0.to_owned()) }
}

impl<T> From<T> for Winstons where T: Into<BigUint> {
    #[inline] fn from(t: T) -> Self { Self(t.into()) }
}

impl AsRef<Winstons> for Winstons {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl Absorbable for Winstons {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        s.absorb(&self.0.to_str_radix(10).into_bytes())
    }
}

pub mod winstons_as_strings {
    use super::*;
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Winstons, D::Error> {
        struct WinstonsVisitor;
        impl<'de> de::Visitor<'de> for WinstonsVisitor {
            type Value = Winstons;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a non-negative amount of Winstons")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Winstons::decode(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(WinstonsVisitor)
    }

    pub fn serialize<S: Serializer>(w: &Winstons, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&w.0.to_str_radix(10))
    }
}

pub mod winstons_as_numbers {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(_deserializer: D) -> Result<Winstons, D::Error> {
        unimplemented!()
    }

    pub fn serialize<S: Serializer>(w: &Winstons, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(w.0.to_u64().unwrap())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Address(Bytes);

impl Address {
    pub fn new<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        let bs = Bytes::new("address", t).with_expected_length(32)?;
        Ok(Address(bs))
    }

    pub fn encode(&self) -> String {
        self.0.encode()
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        Bytes::decode("address", t).and_then(|bs| bs.with_expected_length(32)).map(Self)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl AsRef<Address> for Address {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl Absorbable for Address {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        s.absorb(&self.0.as_slice())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new_with_expected_length("address", 32)).map(Self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Anchor {
    Block(BlockHash),
    Transaction(Option<TxHash>),
}

impl Absorbable for Anchor {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        match &self {
            Anchor::Block(bh) => bh.squeeze(s),
            Anchor::Transaction(Some(txh)) => txh.squeeze(s),
            Anchor::Transaction(None) => Ok(()),
        }
    }
}

impl<'de> Deserialize<'de> for Anchor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AnchorVisitor;
        impl<'de> de::Visitor<'de> for AnchorVisitor {
            type Value = Anchor;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("anchor")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if s.len() == 0 {
                    Ok(Anchor::Transaction(None))
                } else {
                    BlockHash::deserialize(s.into_deserializer()).map(Anchor::Block)
                        .or_else(|_: E| {
                            TxHash::deserialize(s.into_deserializer())
                                .map(Some).map(Anchor::Transaction)
                        })
                }
            }
        }

        deserializer.deserialize_str(AnchorVisitor)
    }
}

impl Serialize for Anchor {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Anchor::Transaction(Some(txh)) => txh.serialize(s),
            Anchor::Transaction(None) => s.serialize_str(""),
            Anchor::Block(bh) => bh.serialize(s),
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Owner { n: BigNum }

impl Owner {
    pub fn address(&self) -> Result<Address, Error> {
        hash(MessageDigest::sha256(), &self.n.to_vec()).map_err(Error::from)
            .map(|bs| Address(Bytes { thing: "address", bytes: bs.to_vec() }))
    }

    pub fn pubkey(&self) -> Result<Rsa<Public>, Error> {
        // https://github.com/ArweaveTeam/arweave/blob/aef590a2e7fbc2703d47449c121058a77916ce16/src/ar_wallet.erl#L15
        Ok(Rsa::from_public_components(self.n.to_owned()?, BigNum::from_u32(65537)?)?)
    }

    pub fn exponent() -> BigNum {
        BigNum::from_u32(65537).unwrap()
    }

    pub fn from<K: HasPublic>(t: &PKeyRef<K>) -> Result<Self, Error> {
        let t = t.rsa()?;
        if t.e().to_owned()? != Self::exponent() {
            Err(Error::invalid_value("RSA key", "incorrect public exponent"))
        } else {
            Ok(Owner { n: t.n().to_owned()? })
        }
    }

    pub fn clone(&self) -> Result<Self, Error> {
        Ok(Owner { n: self.n.to_owned()? })
    }
}

impl<'de> Deserialize<'de> for Owner {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("owner"))
            .and_then(|bs| {
                BigNum::from_slice(bs.as_slice()).map_err(Error::from).map_err(de::Error::custom)
            })
            .map(|n| Owner { n })
    }
}

impl Absorbable for Owner {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        s.absorb(&self.n.to_vec())
    }
}

impl Serialize for Owner {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let bs = self.n.to_vec();
        let enc = base64::encode_config(&bs, base64::URL_SAFE_NO_PAD);
        s.serialize_str(&enc)
    }
}


#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize)]
pub struct Name(Bytes);

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("tag name")).map(Self)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Name { Name(Bytes { thing: "tag name", bytes: Vec::from(s) }) }
}


#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Value(Bytes);

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("tag value")).map(Self)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value { Value(Bytes { thing: "tag value", bytes: Vec::from(s) }) }
}


#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Tag { name: Name, value: Value }

impl From<(Name, Value)> for Tag {
    fn from(kv: (Name, Value)) -> Tag { Tag { name: kv.0, value: kv.1 } }
}

impl From<(&str, &str)> for Tag {
    fn from(kv: (&str, &str)) -> Tag { Tag { name: Name::from(kv.0), value: Value::from(kv.1) } }
}


#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Tags(Vec<Tag>);

impl Tags {
    pub fn new() -> Tags { Tags(vec![]) }
}

impl From<Vec<Tag>> for Tags {
    fn from(ts: Vec<Tag>) -> Tags { Tags(ts) }
}

impl From<Vec<(&str, &str)>> for Tags {
    fn from(ts: Vec<(&str, &str)>) -> Tags {
        Tags(ts.iter().cloned().map(Tag::from).collect())
    }
}

impl Absorbable for Tags {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        for t in self.0.iter() {
            s.absorb(t.name.0.as_slice())?;
            s.absorb(t.value.0.as_slice())?;
        }
        Ok(())
    }
}


#[derive(Debug, Serialize, PartialEq)]
pub struct Signature(Bytes);

impl Signature {
    pub fn new<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        Ok(Signature(Bytes::new("signature", t)))
    }

    pub fn to_transaction_hash(&self) -> Result<TxHash, Error> {
        hash(MessageDigest::sha256(), &self.0.as_slice()).map_err(Error::from)
            .map(|bs| TxHash(Bytes { thing: "transaction hash", bytes: bs.to_vec() }))
    }
}

impl AsRef<Signature> for Signature {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("signature")).map(Self)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Tx {
    pub id: TxHash,
    pub data: Data,
    #[serde(with = "winstons_as_strings")]
    pub quantity: Winstons,
    #[serde(with = "winstons_as_strings")]
    pub reward: Winstons,
    pub target: EmptyStringAsNone<Address>,
    #[serde(rename = "last_tx")]
    pub anchor: Anchor,
    pub owner: Owner,
    pub tags: Tags,
    pub signature: Signature,
}

impl Absorbable for Tx {
    fn squeeze<S: Sponge>(&self, s: &mut S) -> Result<(), Error> {
        // https://github.com/ArweaveTeam/arweave/blob/d882d8a5880b765cd9a65928eaf7c04ea6aedfea/src/ar_tx.erl#L54
        self.owner.squeeze(s)?;
        if let Some(a) = self.target() { a.squeeze(s)?; }
        self.data.squeeze(s)?;
        self.quantity.squeeze(s)?;
        self.reward.squeeze(s)?;
        self.anchor.squeeze(s)?;
        self.tags.squeeze(s)?;
        Ok(())
    }
}

impl AsRef<Tx> for Tx {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl Tx {
    pub fn target(&self) -> Option<&Address> {
        self.target.as_option_ref()
    }

    pub fn verify(&self) -> Result<bool, Error> {
        let pk = PKey::from_rsa(self.owner.pubkey()?)?;
        let mut v = Verifier::new(&pk)?;
        self.squeeze(&mut v)?;
        v.verify(&self.signature.0.as_slice())
    }
}

pub struct Wallet { key: PKey<Private>, owner: Owner, address: Address  }

impl Wallet {
    pub fn address(&self) -> &Address { &self.address }
    pub fn new() -> Result<Self, Error> {
        let key = PKey::from_rsa(Rsa::generate_with_e(4096, &Owner::exponent())?)?;
        let owner = Owner::from(&key)?;
        let address = owner.address()?;
        Ok(Wallet { key, owner, address })
    }

    pub fn owner(&self) -> &Owner { &self.owner }
    pub fn key(&self) -> &PKeyRef<Private> { self.key.as_ref() }
}

impl AsRef<Wallet> for Wallet {
    #[inline] fn as_ref(&self) -> &Self { self }
}
