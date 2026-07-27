use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn remove_if_condition_not_met() {
    let map = map_alice(5);
    let threshold = 10u64;
    let result = map
        .immediate_tx::<GetOneState>()
        .remove_if(ALICE.into(), move |_k, v, _s| *v > threshold)
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
    assert_eq!(map.len(), 1);
}

#[test]
fn param_remove_if() {
    let map = map_alice_bob(5, 15);
    let threshold = 10u64;
    let result = map
        .immediate_tx::<GetTwoParamU64State>()
        .remove_if(ALICE.into(), move |_k, v, _s| *v > threshold)
        .remove_if(BOB.into(), move |_k, v, _s| *v > threshold)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(5),
            result_b: None
        })
    );
    assert_eq!(map.len(), 1);
}
