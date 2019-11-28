use regiusmark::prelude::*;

mod common;
pub use common::*;

#[test]
fn deny_broadcasted_reward_tx() {
    let minter = TestMinter::new();

    let tx = TxVariant::V0(TxVariantV0::RewardTx(RewardTx {
        base: create_tx_header("0.00000 MARK"),
        to: KeyPair::gen().0.into(),
        rewards: get_asset("1.00000 MARK"),
    }));

    let res = minter.request(RequestBody::Broadcast(tx)).unwrap();
    assert!(res.is_err(), format!("{:?}", res));
    assert_eq!(
        res,
        ResponseBody::Error(net::ErrorKind::TxValidation(verify::TxErr::TxProhibited))
    );
}
