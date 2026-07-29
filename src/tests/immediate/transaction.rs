use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn empty_key_works() {
    let map = empty_map();
    map.insert("".into(), 1);
    assert_eq!(map.get_with(&"".into(), |v| *v), Some(1));
    let result = map
        .immediate_tx::<GetOneState>()
        .modify("".into(), |_k, v, s| {
            *v += 1;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(2) }));
}

#[test]
fn transaction_on_empty_map() {
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
fn mixed_ops_in_one_transaction() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetThreeState>()
        .insert_with(ALICE.into(), |_k, _s| 0)
        .insert_with(BOB.into(), |_k, _s| 0)
        .insert_with(CHUCK.into(), |_k, _s| 0)
        .modify(ALICE.into(), |_k, v, _s| *v = 10)
        .modify(BOB.into(), |_k, v, _s| *v = 20)
        .update(CHUCK.into(), |_k, _v, _s| Some(30))
        .get(ALICE.into(), |_k, v, s| s.results.push(v.copied()))
        .get(BOB.into(), |_k, v, s| s.results.push(v.copied()))
        .get(CHUCK.into(), |_k, v, s| s.results.push(v.copied()))
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetThreeState {
            results: vec![Some(10), Some(20), Some(30)]
        })
    );
}

#[test]
fn chain_many_ops() {
    let map: TxMap<u64, u64> = empty_typed_map();
    for i in 0..5u64 {
        let result = map
            .immediate_tx::<IncrementState>()
            .insert_with(i, |_k, _s| 0)
            .execute();
        assert_eq!(result, TxResult::Completed(IncrementState {}));
    }
    assert_eq!(map.len(), 5);
}

#[test]
fn chain_many_ops_with_params() {
    let map = empty_map();
    let p = vec![10u64, 20u64];
    let p2 = p.clone();
    let result = map
        .immediate_tx::<GetVecParamState>()
        .insert_with(ALICE.into(), |_k, _p| 0)
        .insert_with(BOB.into(), |_k, _p| 0)
        .modify(ALICE.into(), move |_k, v, s| {
            *v = p[0];
            s.results.push(Some(*v));
        })
        .modify(BOB.into(), move |_k, _v, s| {
            s.results.push(Some(p2[1]));
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetVecParamState {
            results: vec![Some(10), Some(20)]
        })
    );
}

#[test]
fn chained_modify_and_get() {
    let map: TxMap<String, Counter> = empty_typed_map();
    let result = map
        .immediate_tx::<GetCounterState>()
        .insert_with("ctr".into(), |_k, _s| Counter::default())
        .modify("ctr".into(), |_k, c, _s| c.value += 1)
        .modify("ctr".into(), |_k, c, _s| c.value += 1)
        .get("ctr".into(), |_k, c, s| {
            s.result = c.as_ref().map(|c| c.value);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetCounterState { result: Some(2) })
    );
}

#[test]
fn chained_ops_on_multiple_keys() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetTwoState>()
        .insert_with(ALICE.into(), |_k, _s| 0)
        .insert_with(BOB.into(), |_k, _s| 0)
        .modify(ALICE.into(), |_k, v, _s| *v += 10)
        .modify(BOB.into(), |_k, v, _s| *v += 20)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(10),
            result_b: Some(20)
        })
    );
}

#[test]
fn param_transaction_basic() {
    let map = map_alice(0);
    let param = 50u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += param;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneParamU64State { result: Some(50) })
    );
    let param2 = 30u64;
    let result2 = map
        .immediate_tx::<GetOneParamU64State>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += param2;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result2,
        TxResult::Completed(GetOneParamU64State { result: Some(80) })
    );
}
