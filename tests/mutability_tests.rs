mod loom;
mod fresh;
use arweaver::{Winstons, Wallet, Anchor, TxBuilder};

#[test]
fn faucet() {
    let c = arweaver::Client::new().unwrap();
    let l = loom::Client::new().unwrap();

    let a = fresh::address();
    let q = fresh::quantity();
    assert_eq!(c.balance(&a).unwrap(), Winstons::from(0u32));
    let _txh = l.faucet(&a, &q).unwrap();
    assert_eq!(c.balance(a).unwrap(), q);
}

#[test]
fn tx() {
    let c = arweaver::Client::new().unwrap();
    let l = loom::Client::new().unwrap();

    let w = Wallet::new().unwrap();

    let a = fresh::address();
    let q = fresh::quantity();
    let r = c.price(Some(&a), 0).unwrap();
    let _txh = l.faucet(w.address(), &q + &r).unwrap();

    assert_eq!(c.balance(w.address()).unwrap(), &q + &r);
    assert_eq!(c.balance(&a).unwrap(), Winstons::from(0u32));

    let tx = TxBuilder::new(Anchor::Transaction(None))
        .quantity(q.to_owned()).target(a.to_owned())
        .reward(&c).unwrap().sign(&w).unwrap();

    c.submit(&tx).unwrap();
    let tx0 = l.wait(&tx.id).unwrap();
    assert_eq!(tx, tx0);

    assert_eq!(c.balance(w.address()).unwrap(), Winstons::from(0u32));
    assert_eq!(c.balance(&a).unwrap(), q);
}
