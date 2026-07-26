use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn empty_key_works() {
    let map = empty_map();
    map.insert("".into(), 1);
    assert_eq!(map.get_with(&"".into(), |v| *v), Some(1));
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v += 1)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: "".into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(2) })
    );
}

#[test]
fn transaction_on_empty_map() {
    let map = empty_map();
    let result = map
        .prepare_transaction(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v = 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction()
        .execute(GetOneKeys { key: ALICE.into() }, GetOneParams {});
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: None })
    );
}

#[test]
fn mixed_ops_in_one_transaction() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetThree::SCHEMA)
        .insert_default(GetThree::a)
        .insert_default(GetThree::b)
        .insert_default(GetThree::c)
        .modify(GetThree::a, |_k, v, _p, _s| *v = 10)
        .modify(GetThree::b, |_k, v, _p, _s| *v = 20)
        .update(GetThree::c, |_k, _v, _p, _s| Some(30))
        .get(GetThree::a, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .get(GetThree::b, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .get(GetThree::c, |_k, v, _p, s| {
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
            results: vec![Some(10), Some(20), Some(30)]
        })
    );
}

#[test]
fn chain_many_ops() {
    let map: TxMap<u64, u64> = empty_typed_map();
    for i in 0..5u64 {
        let tx = map
            .prepare_transaction(&Increment::SCHEMA)
            .insert_default(Increment::k)
            .into_transaction();
        assert_eq!(
            tx.execute(IncrementKeys { k: i }, IncrementParams {}),
            TxResult::Completed(IncrementState {})
        );
    }
    assert_eq!(map.len(), 5);
}

#[test]
fn chain_many_ops_with_params() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetVecParam::SCHEMA)
        .insert_default(GetVecParam::a)
        .insert_default(GetVecParam::b)
        .modify(GetVecParam::a, |_k, v, p, _s| *v = p.param[0])
        .modify(GetVecParam::b, |_k, v, p, _s| *v = p.param[1])
        .get(GetVecParam::a, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .get(GetVecParam::b, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .into_transaction();
    let result = tx.execute(
        GetVecParamKeys {
            a: ALICE.into(),
            b: BOB.into(),
        },
        GetVecParamParams {
            param: vec![10, 20],
        },
    );
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
    let tx = map
        .prepare_transaction(&GetCounter::SCHEMA)
        .insert_default(GetCounter::key)
        .modify(GetCounter::key, |_k, c, _p, _s| c.value += 1)
        .modify(GetCounter::key, |_k, c, _p, _s| c.value += 1)
        .get(GetCounter::key, |_k, c, _p, s| {
            s.result = c.as_ref().map(|c| c.value);
        })
        .into_transaction();
    let result = tx.execute(
        GetCounterKeys {
            key: "ctr".into(),
        },
        GetCounterParams {},
    );
    assert_eq!(
        result,
        TxResult::Completed(GetCounterState { result: Some(2) })
    );
}

#[test]
fn chained_ops_on_multiple_keys() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetTwo::SCHEMA)
        .insert_default(GetTwo::a)
        .insert_default(GetTwo::b)
        .modify(GetTwo::a, |_k, v, _p, _s| *v += 10)
        .modify(GetTwo::b, |_k, v, _p, _s| *v += 20)
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
            result_a: Some(10),
            result_b: Some(20)
        })
    );
}
