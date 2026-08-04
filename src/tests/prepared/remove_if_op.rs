use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn remove_if() {
    let map = map_alice_bob(1, 2);
    let tx = map
        .prepared_tx(&RemoveMultiple::SCHEMA)
        .remove_if(RemoveMultiple::a, |k, v, _p, s| {
            let cond = *v >= 2;
            if cond {
                s.user.push(Some(k.clone()));
            } else {
                s.user.push(None);
            }
            cond
        })
        .remove_if(RemoveMultiple::b, |k, v, _p, s| {
            let cond = *v >= 2;
            if cond {
                s.user.push(Some(k.clone()));
            } else {
                s.user.push(None);
            }
            cond
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            RemoveMultipleKeys {
                a: ALICE.into(),
                b: BOB.into()
            },
            RemoveMultipleParams {}
        ),
        TxResult::Completed {
            state: RemoveMultipleState {
                user: vec![None, Some(BOB.into())]
            }
        }
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn param_remove_if() {
    let map = map_alice_bob(5, 15);
    let tx = map
        .prepared_tx(&GetTwoParamU64::SCHEMA)
        .remove_if(GetTwoParamU64::a, |_k, v, p, _s| *v > p.param)
        .remove_if(GetTwoParamU64::b, |_k, v, p, _s| *v > p.param)
        .get(GetTwoParamU64::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetTwoParamU64Keys {
                a: ALICE.into(),
                b: BOB.into()
            },
            GetTwoParamU64Params { param: 10 }
        ),
        TxResult::Completed {
            state: GetTwoParamU64State {
                result_a: Some(5),
                result_b: None
            }
        }
    );
    assert_eq!(map.len(), 1);
}
