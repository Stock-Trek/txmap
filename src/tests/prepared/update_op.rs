use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn update_overwrites_when_returning_some() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .update(GetOne::key, |_k, _v, _p, _s| Some(42))
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn update_removes_when_returning_none() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .update(GetOne::key, |_k, _v, _p, _s| None)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: None })
    );
}

#[test]
fn update_transforms_existing_value() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .update(GetOne::key, |_k, v, _p, _s| v.map(|x| x * 2))
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(2) })
    );
}

#[test]
fn update_peek_modifies_based_on_peek() {
    let map = map_alice_bob(10, 5);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .update(GetTwo::a, |_k, v, _p, _s| v.map(|x| x + 5))
        .get(GetTwo::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetTwoKeys {
                a: ALICE.into(),
                b: BOB.into()
            },
            GetTwoParams {}
        ),
        TxResult::Completed(GetTwoState {
            result_a: Some(15),
            result_b: None
        })
    );
}

#[test]
fn param_map_op() {
    let map = map_alice(10);
    let tx = map
        .prepared_tx(&GetOneParamU64::SCHEMA)
        .update(GetOneParamU64::key, |_k, v, p, _s| v.map(|x| x * p.param))
        .get(GetOneParamU64::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys { key: ALICE.into() },
            GetOneParamU64Params { param: 3 }
        ),
        TxResult::Completed(GetOneParamU64State { result: Some(30) })
    );
}

#[test]
fn param_update_peek() {
    let map = map_alice_bob(10, 5);
    let tx = map
        .prepared_tx(&GetTwoParamU64::SCHEMA)
        .update(GetTwoParamU64::a, |_k, v, p, _s| {
            v.map(|x| (x + 5) * p.param)
        })
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
            GetTwoParamU64Params { param: 2 }
        ),
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(30),
            result_b: None
        })
    );
}
