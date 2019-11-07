use arweave::*;
mod settings;

#[test]
fn info() {
    let c = Client::new().unwrap();
    let i = c.info().unwrap();
    assert!(i.height > Height::from(316893));
}

#[test]
fn block() {
    let c = Client::new().unwrap();
    let i = c.info().unwrap();
    let b0 = c.block(&i.current).unwrap();
    assert_eq!(b0.indep, i.current);
    assert_eq!(i.height, b0.height);

    let b1 = c.block(&b0.previous_block).unwrap();
    assert_eq!(b1.indep, b0.previous_block);
    assert_eq!(b1.height + Height::from(1), b0.height);
}

#[test]
fn current_block() {
    let c = Client::new().unwrap();
    let b0 = c.current_block().unwrap();
    let b1 = c.block(&b0.indep).unwrap();
    assert_eq!(b0.indep, b1.indep);
    assert_eq!(b0.height, b1.height);
}

#[test]
fn height() {
    let c = Client::new().unwrap();
    let i = c.info().unwrap();

    let b0 = c.height(i.height).unwrap();
    assert_eq!(b0.indep, i.current);
    assert_eq!(b0.height, i.height);

    let b1 = c.height(i.height - Height::from(1)).unwrap();
    assert_eq!(b1.indep, b0.previous_block);
    assert_eq!(b1.height + Height::from(1), b0.height);
}

#[test]
fn txs() {
    let c = Client::new().unwrap();
    let txs = c.block(settings::block_with_transactions()).unwrap().txs;
    assert!(txs.len() > 0);
    for txh in txs {
        let tx = c.tx(&txh).unwrap();
        assert_eq!(txh, tx.id);
    }
}
