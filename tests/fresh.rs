extern crate rand;

use arweave::{Address};

pub fn address() -> Address {
    Address::new(rand::random::<[u8; 32]>()).unwrap()
}
