extern crate rand;

use arweave::{Address, Winstons};

pub fn address() -> Address {
    Address::new(rand::random::<[u8; 32]>()).unwrap()
}

pub fn quantity() -> Winstons {
    let i = Winstons::from(rand::random::<u64>() % 100000000000);
    Winstons::from(i)
}
