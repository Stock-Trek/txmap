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
        TxResult::Completed {
            state: GetOneState { result: Some(42) }
        }
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
        TxResult::Completed {
            state: GetOneState { result: None }
        }
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
        TxResult::Completed {
            state: GetOneState { result: Some(2) }
        }
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
        TxResult::Completed {
            state: GetOneParamU64State { result: Some(30) }
        }
    );
}
