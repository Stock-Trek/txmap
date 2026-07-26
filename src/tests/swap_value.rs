use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn swap_value_exchanges_values() {
    let map = map_alice_bob(1, 2);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .swap_value(GetTwo::a, GetTwo::b)
        .get(GetTwo::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .get(GetTwo::b, |_k, v, _p, s| {
            s.result_b = v.copied();
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
            result_a: Some(2),
            result_b: Some(1)
        })
    );
}

#[test]
fn swap_with_missing_value() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .swap_value(GetTwo::a, GetTwo::b)
        .get(GetTwo::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .get(GetTwo::b, |_k, v, _p, s| {
            s.result_b = v.copied();
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
            result_a: None,
            result_b: Some(1)
        })
    );
}

#[test]
fn swap_value_same_key() {
    let map = map_alice(7);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .swap_value(GetOne::key, GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(7) })
    );
}
