mod loom;
mod fresh;
use arweave::{Winstons};

#[test]
fn faucet() {
    let c = arweave::Client::new().unwrap();
    let l = loom::Client::new().unwrap();

    let a = fresh::address();
    let q = fresh::quantity();
    assert_eq!(c.balance(&a).unwrap(), Winstons::from(0u32));
    let _txh = l.faucet(&a, &q).unwrap();
    assert_eq!(c.balance(a).unwrap(), q);
}
