use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn param_get_many() {
    let map = map_alice(10);
    let tx = map
        .prepared_tx(&GetTwoParam::SCHEMA)
        .modify(GetTwoParam::a, |_k, v, _p, _s| *v += 0)
        .modify(GetTwoParam::b, |_k, v, _p, _s| *v += 0)
        .get(GetTwoParam::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .get(GetTwoParam::b, |_k, v, _p, s| {
            s.result_b = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetTwoParamKeys {
                a: ALICE.into(),
                b: BOB.into()
            },
            GetTwoParamParams { _p: () }
        ),
        TxResult::Completed {
            state: GetTwoParamState {
                result_a: Some(10),
                result_b: None
            }
        }
    );
}
