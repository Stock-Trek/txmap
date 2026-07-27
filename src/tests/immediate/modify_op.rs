use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn modify_existing_key() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += 5;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(6) }));
}

#[test]
fn modify_missing_key_is_noop() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v = 42;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
}

#[test]
fn modify_peek_existing_key() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 5)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(6),
            result_b: None
        })
    );
}

#[test]
fn modify_peek_missing_key_is_noop() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify(ALICE.into(), |_k, v, _s| *v = 42)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: None,
            result_b: None
        })
    );
}

#[test]
fn modify_peek_can_use_peeked_values() {
    let map = map_alice_bob(1, 2);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 2)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
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
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v = 99;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(99) })
    );
}

#[test]
fn modify_peek_modifies_with_peek_values() {
    let map = empty_map();
    map.insert("target".into(), 100);
    map.insert("reference".into(), 50);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify("target".into(), |_k, v, _s| *v += 50)
        .get("target".into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
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
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify("missing".into(), |_k, v, _s| *v = 0)
        .get("missing".into(), |_k, v, s| s.result_a = v.copied())
        .get("ref".into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
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
    let result = map
        .immediate_tx::<GetThreeState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 20 + 3)
        .get(ALICE.into(), |_k, v, s| s.results.push(v.copied()))
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetThreeState {
            results: vec![Some(123)]
        })
    );
}

#[test]
fn param_modify_peek() {
    let map = map_alice_bob(10, 5);
    let factor = 3u64;
    let result = map
        .immediate_tx::<GetTwoParamU64State>()
        .modify(ALICE.into(), move |_k, v, s| {
            *v = 5 * factor;
            s.result_a = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(15),
            result_b: None
        })
    );
}
