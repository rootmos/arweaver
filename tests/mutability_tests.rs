mod loom;

#[test]
fn faucet() {
    let c = arweave::Client::new().unwrap();
    let l = loom::Client::new().unwrap();
}
