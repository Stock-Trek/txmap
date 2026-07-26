use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn modify_existing_key() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v += 5)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(6) })
    );
}

#[test]
fn modify_missing_key_is_noop() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v = 42)
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
fn modify_peek_existing_key() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .modify(GetTwo::a, |_k, v, _p, _s| *v += 5)
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
            result_a: Some(6),
            result_b: None
        })
    );
}

#[test]
fn modify_peek_missing_key_is_noop() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .modify(GetTwo::a, |_k, v, _p, _s| *v = 42)
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
            result_a: None,
            result_b: None
        })
    );
}

#[test]
fn modify_peek_can_use_peeked_values() {
    let map = map_alice_bob(1, 2);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .modify(GetTwo::a, |_k, v, _p, _s| *v += 2)
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
            result_a: Some(3),
            result_b: None
        })
    );
}

#[test]
fn modify_peek_with_empty_peek_keys() {
    let map = empty_map();
    map.insert(ALICE.into(), 10);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v = 99)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(99) })
    );
}

#[test]
fn modify_peek_modifies_with_peek_values() {
    let map = empty_map();
    map.insert("target".into(), 100);
    map.insert("reference".into(), 50);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .modify(GetTwo::a, |_k, v, _p, _s| {
            *v += 50; // we use the b value inline instead of peek
        })
        .get(GetTwo::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetTwoKeys {
                a: "target".into(),
                b: "reference".into()
            },
            GetTwoParams {}
        ),
        TxResult::Completed(GetTwoState {
            result_a: Some(150),
            result_b: None
        })
    );
}

#[test]
fn modify_peek_missing_target_is_noop() {
    let map = empty_map();
    map.insert("ref".into(), 99);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .modify(GetTwo::a, |_k, v, _p, _s| *v = 0)
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
                a: "missing".into(),
                b: "ref".into()
            },
            GetTwoParams {}
        ),
        TxResult::Completed(GetTwoState {
            result_a: None,
            result_b: Some(99)
        })
    );
}

#[test]
fn modify_peek_modifies_using_peeked_values() {
    let map = empty_map();
    map.insert(ALICE.into(), 100);
    map.insert(BOB.into(), 20);
    map.insert(CHUCK.into(), 3);
    let tx = map
        .prepared_tx(&GetThree::SCHEMA)
        .modify(GetThree::a, |_k, v, _p, _s| {
            *v += 20 + 3; // bob(20) + chuck(3)
        })
        .get(GetThree::a, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetThreeKeys {
                a: ALICE.into(),
                b: BOB.into(),
                c: CHUCK.into()
            },
            GetThreeParams {}
        ),
        TxResult::Completed(GetThreeState {
            results: vec![Some(123)]
        })
    );
}
