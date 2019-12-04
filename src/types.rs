use std::fmt;
use std::convert::From;
use std::marker::PhantomData;

use crate::error::Error;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use openssl::bn::BigNum;
use openssl::hash::{MessageDigest, hash};
use openssl::rsa::{Rsa, Padding};
use openssl::pkey::{PKey, PKeyRef, Public};
use serde::de;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{IntoDeserializer};

#[derive(Debug, Clone, Copy)]
struct EmptyStringAsNone<T>(Option<T>);

impl<T> EmptyStringAsNone<T> {
    fn as_option_ref(&self) -> Option<&T> { self.0.as_ref() }
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


#[derive(Debug, PartialEq, Eq, Clone)]
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


#[derive(Debug, PartialEq, Eq, Clone)]
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

impl<'de> Deserialize<'de> for TxHash {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new_with_expected_length("transaction hash", 32)).map(Self)
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct Data(Bytes);

impl Data {
    pub fn len(&self) -> usize { self.0.len() }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        Bytes::decode("data", t).map(Self)
    }
}

impl From<Vec<u8>> for Data {
    fn from(bytes: Vec<u8>) -> Data { Data(Bytes { thing: "data", bytes }) }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("data")).map(Self)
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl<T> From<T> for Winstons where T: Into<BigUint> {
    #[inline] fn from(t: T) -> Self { Self(t.into()) }
}

impl AsRef<Winstons> for Winstons {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl<'de> Deserialize<'de> for Winstons {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
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
}

impl Serialize for Winstons {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(self.0.to_u64().unwrap())
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
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

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new_with_expected_length("address", 32)).map(Self)
    }
}

impl Serialize for Address {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.encode())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Anchor {
    Block(BlockHash),
    Transaction(Option<TxHash>),
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


#[derive(Debug)]
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Name(Bytes);

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("tag name")).map(Self)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Name { Name(Bytes { thing: "tag name", bytes: Vec::from(s) }) }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Value(Bytes);

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("tag value")).map(Self)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value { Value(Bytes { thing: "tag value", bytes: Vec::from(s) }) }
}


#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Tag { name: Name, value: Value }

impl From<(Name, Value)> for Tag {
    fn from(kv: (Name, Value)) -> Tag { Tag { name: kv.0, value: kv.1 } }
}

impl From<(&str, &str)> for Tag {
    fn from(kv: (&str, &str)) -> Tag { Tag { name: Name::from(kv.0), value: Value::from(kv.1) } }
}


#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug)]
pub struct Signature(Bytes);

impl AsRef<Signature> for Signature {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_str(BytesVisitor::new("signature")).map(Self)
    }
}


#[derive(Deserialize, Debug)]
pub struct Tx {
    pub id: TxHash,
    pub data: Data,
    pub quantity: Winstons,
    pub reward: Winstons,
    target: EmptyStringAsNone<Address>,
    #[serde(rename = "last_tx")]
    pub anchor: Anchor,
    pub owner: Owner,
    pub tags: Tags,
    pub signature: Signature,
}


trait Sponge {
    fn absorb<T: AsRef<[u8]>>(&mut self, t: T) -> Result<(), Error>;
}

struct Verifier<'a> { v: openssl::sign::Verifier<'a> }

impl<'a> Verifier<'a> {
    fn new(pk: &'a PKeyRef<Public>) -> Result<Verifier<'a>, Error> {
        let mut v = openssl::sign::Verifier::new_without_digest(pk)?;
        v.set_rsa_padding(Padding::PKCS1_PSS)?;
        Ok(Verifier { v })
    }

    fn verify<S: AsRef<Signature>>(self, sig: S) -> Result<bool, Error> {
        Ok(self.v.verify(sig.as_ref().0.as_slice())?)
    }
}

impl Sponge for Verifier<'_> {
    fn absorb<T: AsRef<[u8]>>(&mut self, t: T) -> Result<(), Error> {
        self.v.update(t.as_ref()).map_err(Error::from)
    }
}


impl Tx {
    pub fn target(&self) -> Option<&Address> {
        self.target.as_option_ref()
    }

    fn signature_data<S: Sponge>(&self, mut s: S) -> Result<S, Error>
    {
        // https://github.com/ArweaveTeam/arweave/blob/d882d8a5880b765cd9a65928eaf7c04ea6aedfea/src/ar_tx.erl#L54
        s.absorb(&self.owner.n.to_vec())?;

        if let Some(Address(a)) = self.target() {
            s.absorb(a.as_slice())?
        }

        s.absorb(self.data.0.as_slice())?;

        s.absorb(&self.quantity.0.to_str_radix(10).into_bytes())?;
        s.absorb(&self.reward.0.to_str_radix(10).into_bytes())?;

        match &self.anchor {
            Anchor::Block(BlockHash(bh)) => s.absorb(bh.as_slice())?,
            Anchor::Transaction(Some(TxHash(txh))) => s.absorb(txh.as_slice())?,
            Anchor::Transaction(None) => (),
        }

        for t in self.tags.0.iter() {
            s.absorb(t.name.0.as_slice())?;
            s.absorb(t.value.0.as_slice())?;
        }

        Ok(s)
    }

    pub fn verify(&self) -> Result<bool, Error> {
        let pk = PKey::from_rsa(self.owner.pubkey()?)?;
        let v = Verifier::new(&pk)?;
        let v = self.signature_data(v)?;
        v.verify(&self.signature)
    }
}
