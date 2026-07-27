use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn param_swap_value() {
    let map = map_alice_bob(1, 2);
    let result = map
        .immediate_tx::<GetTwoParamState>()
        .swap_value(ALICE.into(), BOB.into())
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamState {
            result_a: Some(2),
            result_b: Some(1)
        })
    );
}
