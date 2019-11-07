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
