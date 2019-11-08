use arweave::*;
use chrono::prelude::*;

pub fn recent_block_height() -> Height {
    Height::from(317621)
}

pub fn recent_block_timestamp() -> DateTime<Utc> {
    Utc.ymd(2019, 11, 7).and_hms(15, 03, 24)
}

pub fn block_with_transactions() -> (BlockHash, DateTime<Utc>)  {
    let bh = BlockHash::decode("TQpzWTuYMv82YPLEeaAKJawJlknA5cDcesHCGVvZFzSFrpfWZxc-tOmLU-lx1B4v").unwrap();
    let ts = Utc.ymd(2019, 11, 7).and_hms(11, 59, 38);
    (bh, ts)
}

pub fn data_transaction() -> (TxHash, Winstons, Anchor) {
    let h = TxHash::decode("et36AGA5eo4HzVNi39nSvTbltzhoRPq643MzzwrH38w").unwrap();
    let r = Winstons::from(42360199u64);
    let a = Anchor::Block(BlockHash::decode("2serU-303rThvozelaLz67ftihCw7cJPAEF40SkPkfyvz92Z5gCKVfhadoVU3ZRg").unwrap());
    (h, r, a)
}

pub fn transfer_transaction() -> (TxHash, Winstons, Winstons, Anchor, Address, Address) {
    let h = TxHash::decode("wbezNDNLwOf7qIYEtyHCjRs_kUafeiGF28REpip6DU8").unwrap();
    let q = Winstons::from(339000000000000u64);
    let r = Winstons::from(321179212u64);
    let a = Anchor::Transaction(Some(TxHash::decode("CCH2h2MzMP7WMh0Xf3GYL7zZDbU7E4CZPJWngp1qmDc").unwrap()));
    let from = Address::decode("_1on1ufdt7Pye8VBn9om-9fBI43kzKYKrYyb_ShfECg").unwrap();
    let to = Address::decode("T22ykpEoUQerm0_8wSpiOE_2xUOGmG1lnf7niSiQlaU").unwrap();
    (h, r, q, a, from, to)
}
