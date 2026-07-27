use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn param_map_op() {
    let map = map_alice(10);
    let factor = 3u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .update(ALICE.into(), move |_k, v, s| {
            let r = v.map(|x| x * factor);
            s.result = r;
            r
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneParamU64State { result: Some(30) })
    );
}

#[test]
fn param_update_peek() {
    let map = map_alice_bob(10, 5);
    let factor = 2u64;
    let result = map
        .immediate_tx::<GetTwoParamU64State>()
        .update(ALICE.into(), move |_k, v, s| {
            let r = v.map(|x| (x + 5) * factor);
            s.result_a = r;
            r
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(30),
            result_b: None
        })
    );
}
