use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn update_overwrites_when_returning_some() {
    let map = map_alice(1);
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
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
        .prepare_transaction(&GetOne::SCHEMA)
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
        .prepare_transaction(&GetOne::SCHEMA)
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
        .prepare_transaction(&GetTwo::SCHEMA)
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
