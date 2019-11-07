use arweave::*;
use chrono::prelude::*;

pub fn recent_block_height() -> Height {
    Height::from(316893)
}

pub fn block_with_transactions() -> (BlockHash, DateTime::<Utc>)  {
    let bh = BlockHash::decode("TQpzWTuYMv82YPLEeaAKJawJlknA5cDcesHCGVvZFzSFrpfWZxc-tOmLU-lx1B4v").unwrap();
    let ts = Utc.ymd(2019, 11, 7).and_hms(11, 59, 38);
    (bh, ts)
}

pub fn data_transaction() -> TxHash {
    TxHash::decode("et36AGA5eo4HzVNi39nSvTbltzhoRPq643MzzwrH38w").unwrap()
}

pub fn transfer_transaction() -> TxHash {
    TxHash::decode("lDNUhC3hKrTny4p6ugLACPyQtXP0f8Rax8v2zfCkmbY").unwrap()
}
