use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn param_transaction_basic() {
    let map = map_alice(0);
    let tx = map
        .prepare_transaction(&GetOneParamU64::SCHEMA)
        .modify(GetOneParamU64::key, |_k, v, p, _s| *v += p.param)
        .get(GetOneParamU64::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys {
                key: ALICE.into()
            },
            GetOneParamU64Params { param: 50 }
        ),
        TxResult::Completed(GetOneParamU64State { result: Some(50) })
    );
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys {
                key: ALICE.into()
            },
            GetOneParamU64Params { param: 30 }
        ),
        TxResult::Completed(GetOneParamU64State { result: Some(80) })
    );
}

#[test]
fn param_requirement_not_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 100);
    let tx = map
        .prepare_transaction(&GetOneParamU64::SCHEMA)
        .require(
            "sufficient",
            GetOneParamU64::key,
            |v, p, _s| v.copied().unwrap_or(0) >= p.param,
        )
        .modify(GetOneParamU64::key, |_k, v, _p, _s| *v += 0)
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys {
                key: "funds".into()
            },
            GetOneParamU64Params { param: 50 }
        ),
        TxResult::Completed(GetOneParamU64State { result: None })
    );
    assert!(matches!(
        tx.execute(
            GetOneParamU64Keys {
                key: "funds".into()
            },
            GetOneParamU64Params { param: 200 }
        ),
        TxResult::RequirementNotMet(0, _)
    ));
}

#[test]
fn param_insert_with() {
    let map: TxMap<String, String> = empty_typed_map();
    let tx = map
        .prepare_transaction(&GetOneParamString::SCHEMA)
        .insert_with(GetOneParamString::key, |_k, p, _s| p.param.clone())
        .get(GetOneParamString::key, |_k, v, _p, s| {
            s.result = v.cloned();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamStringKeys {
                key: ALICE.into()
            },
            GetOneParamStringParams {
                param: "hello".into()
            }
        ),
        TxResult::Completed(GetOneParamStringState {
                result: Some("hello".into())
            }
        )
    );
}

#[test]
fn param_map_op() {
    let map = map_alice(10);
    let tx = map
        .prepare_transaction(&GetOneParamU64::SCHEMA)
        .update(GetOneParamU64::key, |_k, v, p, _s| v.map(|x| x * p.param))
        .get(GetOneParamU64::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys {
                key: ALICE.into()
            },
            GetOneParamU64Params { param: 3 }
        ),
        TxResult::Completed(GetOneParamU64State { result: Some(30) })
    );
}

#[test]
fn param_remove_where() {
    let map = map_alice_bob(5, 15);
    let tx = map
        .prepare_transaction(&GetTwoParamU64::SCHEMA)
        .remove_where(GetTwoParamU64::a, |_k, v, p, _s| *v > p.param)
        .remove_where(GetTwoParamU64::b, |_k, v, p, _s| *v > p.param)
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
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(5),
            result_b: None
        })
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn param_modify_peek() {
    let map = map_alice_bob(10, 5);
    let tx = map
        .prepare_transaction(&GetTwoParamU64::SCHEMA)
        .modify(GetTwoParamU64::a, |_k, v, p, _s| {
            *v = 5 * p.param // bob's value (5) * param
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
            GetTwoParamU64Params { param: 3 }
        ),
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(15),
            result_b: None
        })
    );
}

#[test]
fn param_swap_value() {
    let map = map_alice_bob(1, 2);
    let tx = map
        .prepare_transaction(&GetTwoParam::SCHEMA)
        .swap_value(GetTwoParam::a, GetTwoParam::b)
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
        TxResult::Completed(GetTwoParamState {
            result_a: Some(2),
            result_b: Some(1)
        })
    );
}

#[test]
fn param_move_value() {
    let map = map_alice(42);
    let tx = map
        .prepare_transaction(&GetTwoParam::SCHEMA)
        .move_value(GetTwoParam::a, GetTwoParam::b)
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
        TxResult::Completed(GetTwoParamState {
            result_a: None,
            result_b: Some(42)
        })
    );
}

#[test]
fn param_get_all() {
    let map = map_alice(10);
    let tx = map
        .prepare_transaction(&GetTwoParam::SCHEMA)
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
        TxResult::Completed(GetTwoParamState {
            result_a: Some(10),
            result_b: None
        })
    );
}

#[test]
fn param_insert_default() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_default(GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn param_update_peek() {
    let map = map_alice_bob(10, 5);
    let tx = map
        .prepare_transaction(&GetTwoParamU64::SCHEMA)
        .update(GetTwoParamU64::a, |_k, v, p, _s| {
            v.map(|x| (x + 5) * p.param) // bob's value (5) is hardcoded
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
