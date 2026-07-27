use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn param_move_value() {
    let map = map_alice(42);
    let result = map
        .immediate_tx::<GetTwoParamState>()
        .move_value(ALICE.into(), BOB.into())
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamState {
            result_a: None,
            result_b: Some(42)
        })
    );
}
