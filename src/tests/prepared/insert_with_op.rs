use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_with_creates_entry() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
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
fn insert_with_overwrites_existing() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
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
fn param_insert_with() {
    let map: TxMap<String, String> = empty_typed_map();
    let tx = map
        .prepared_tx(&GetOneParamString::SCHEMA)
        .insert_with(GetOneParamString::key, |_k, p, _s| p.param.clone())
        .get(GetOneParamString::key, |_k, v, _p, s| {
            s.result = v.cloned();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamStringKeys { key: ALICE.into() },
            GetOneParamStringParams {
                param: "hello".into()
            }
        ),
        TxResult::Completed {
            state: GetOneParamStringState {
                result: Some("hello".into())
            }
        }
    );
}
