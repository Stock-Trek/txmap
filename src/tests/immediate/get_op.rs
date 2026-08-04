use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn chain_get_only() {
    let map = map_alice_bob(10, 20);
    let result = map
        .immediate_tx::<GetTwoState>()
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetTwoState {
                result_a: Some(10),
                result_b: Some(20)
            }
        }
    );
}

#[test]
fn param_get_many() {
    let map = map_alice(10);
    let result = map
        .immediate_tx::<GetTwoParamState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 0)
        .modify(BOB.into(), |_k, v, _s| *v += 0)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetTwoParamState {
                result_a: Some(10),
                result_b: None
            }
        }
    );
}
