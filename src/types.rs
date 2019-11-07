use std::fmt;
use std::convert::From;
use std::marker::PhantomData;

use crate::error::Error;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use serde::de;
use serde::{Deserialize, Deserializer};
use serde::de::{IntoDeserializer};

#[derive(Debug, Clone, Copy)]
pub struct EmptyStringAsNone<T>(Option<T>);

impl<T> EmptyStringAsNone<T> {
    pub fn as_option(self) -> Option<T> { self.0 }
    pub fn as_option_ref(&self) -> Option<&T> { self.0.as_ref() }
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
            T::deserialize(s.into_deserializer()).map(|t| EmptyStringAsNone(Some(t)))
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

#[derive(PartialEq, Eq, Clone)]
pub struct BlockHash(Vec<u8>);

impl BlockHash {
    pub fn encode(&self) -> String {
        base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        let mut bh = BlockHash(vec![0; 48]);
        match base64::decode_config_slice(&t, base64::URL_SAFE_NO_PAD, &mut bh.0) {
            Ok(48) => Ok(bh),
            Ok(_) => Err(Error::invalid_value(
                    "block hash", "invalid length (should be 48 bytes)")),
            Err(_) => Err(Error::invalid_value(
                    "block hash", "invalid format (should be base64 URL-safe without padding)")),
        }
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

impl AsRef<BlockHash> for BlockHash {
    #[inline] fn as_ref(&self) -> &Self { self }
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
                BlockHash::decode(v).map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(BlockHashVisitor)
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

#[derive(PartialEq, Eq, Clone)]
pub struct TxHash(Vec<u8>);

impl TxHash {
    pub fn encode(&self) -> String {
        base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        let mut bh = TxHash(vec![0; 32]);
        match base64::decode_config_slice(&t, base64::URL_SAFE_NO_PAD, &mut bh.0) {
            Ok(32) => Ok(bh),
            Ok(_) => Err(Error::invalid_value(
                    "transaction hash", "invalid length (should be 32 bytes)")),
            Err(_) => Err(Error::invalid_value(
                    "transaction hash", "invalid format (should be base64 URL-safe without padding)")),
        }
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl fmt::Debug for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxHash({})", self.encode())
    }
}

impl AsRef<TxHash> for TxHash {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl<'de> Deserialize<'de> for TxHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TxHashVisitor;
        impl<'de> de::Visitor<'de> for TxHashVisitor {
            type Value = TxHash;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("transaction hash")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                TxHash::decode(v).map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(TxHashVisitor)
    }
}


#[derive(Debug)]
pub struct Data(Vec<u8>);

impl Data {
    pub fn len(&self) -> usize { self.0.len() }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        base64::decode_config(&t, base64::URL_SAFE_NO_PAD).map(Self).map_err(|_| {
            Error::invalid_value("block hash",
                                 "invalid format (should be base64 URL-safe without padding)")
        })
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DataVisitor;
        impl<'de> de::Visitor<'de> for DataVisitor {
            type Value = Data;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("data")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Data::decode(v).map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(DataVisitor)
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

impl<'de> Deserialize<'de> for Winstons {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WinstonsVisitor;
        impl<'de> de::Visitor<'de> for WinstonsVisitor {
            type Value = Winstons;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a non-negative amount of Winstons")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Winstons::decode(v).map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(WinstonsVisitor)
    }
}


#[derive(PartialEq, Eq, Clone)]
pub struct Address(Vec<u8>);

impl Address {
    pub fn encode(&self) -> String {
        base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        let mut a = Address(vec![0; 32]);
        match base64::decode_config_slice(&t, base64::URL_SAFE_NO_PAD, &mut a.0) {
            Ok(32) => Ok(a),
            Ok(_) => Err(Error::invalid_value(
                    "address", "invalid length (should be 32 bytes)")),
            Err(_) => Err(Error::invalid_value(
                    "address", "invalid format (should be base64 URL-safe without padding)")),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self.encode())
    }
}

impl AsRef<Address> for Address {
    #[inline] fn as_ref(&self) -> &Self { self }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AddressVisitor;
        impl<'de> de::Visitor<'de> for AddressVisitor {
            type Value = Address;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("address")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Address::decode(v).map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(AddressVisitor)
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


#[derive(Deserialize, Debug)]
pub struct Tx {
    pub id: TxHash,
    pub data: Data,
    pub quantity: Winstons,
    pub reward: Winstons,
    target: EmptyStringAsNone<Address>,
    pub last_tx: Anchor,
}

impl Tx {
    pub fn target(&self) -> Option<&Address> {
        self.target.as_option_ref()
    }
}
