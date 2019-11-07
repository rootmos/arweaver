use std::fmt;

use crate::error::Error;

use chrono::{DateTime, Utc};

use serde::{Deserialize, Deserializer};
use serde::de;

#[derive(PartialEq, Eq, Clone)]
pub struct BlockHash(Vec<u8>);

impl BlockHash {
    pub fn encode(&self) -> String {
        base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
    }

    pub fn decode<T: AsRef<[u8]>>(t: T) -> Result<Self, Error> {
        base64::decode_config(&t, base64::URL_SAFE_NO_PAD)
            .map(Self).map_err(Error::from)
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


#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Height(u64);

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<u64> for Height {
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

#[derive(PartialEq, Eq, Clone)]
pub struct TxHash(Vec<u8>);

impl TxHash {
    pub fn encode(&self) -> String {
        base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

impl fmt::Debug for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockHash({})", self.encode())
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
                let mut txh = TxHash(vec![0; 32]);
                match base64::decode_config_slice(v, base64::URL_SAFE_NO_PAD, &mut txh.0) {
                    Ok(32) => Ok(txh),
                    _ => Err(de::Error::custom("should be 32 bytes base64 URL-safe encoded"))
                }
            }
        }

        deserializer.deserialize_str(TxHashVisitor)
    }
}


#[derive(Deserialize, Debug)]
pub struct Tx {
    pub id: TxHash,
}
