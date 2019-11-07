use std::fmt;
use std::convert::From;

use crate::error::Error;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use serde::de;
use serde::{Deserialize, Deserializer};

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
    #[inline] fn as_ref(&self) -> &BlockHash { self }
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
    #[inline] fn as_ref(&self) -> &Height { self }
}


#[derive(Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "indep_hash")]
    pub indep: BlockHash,
    pub previous_block: BlockHash,
    pub height: Height,
    pub txs: Vec<TxHash>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
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
    #[inline] fn as_ref(&self) -> &TxHash { self }
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


#[derive(Deserialize, Debug)]
pub struct Tx {
    pub id: TxHash,
    pub data: Data,
    pub quantity: Winstons,
    pub reward: Winstons,
}
