use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn get_or_insert_creates_entry() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .get_or_insert(GetOne::key, 42, |_k, v, _p, s| {
            s.result = Some(*v);
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(42) }
        }
    );
    assert_eq!(map.get_copied(&ALICE.into()), Some(42));
}

#[test]
fn get_or_insert_does_not_overwrite_existing() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .get_or_insert(GetOne::key, 42, |_k, v, _p, s| {
            s.result = Some(*v);
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(1) }
        }
    );
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

#[test]
fn get_or_insert_is_reusable() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .get_or_insert(GetOne::key, 42, |_k, v, _p, s| {
            s.result = Some(*v);
        })
        .into_transaction();
    let first = tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {});
    let second = tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {});
    assert_eq!(first, second);
    assert_eq!(
        first,
        TxResult::Completed {
            state: GetOneState { result: Some(42) }
        }
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn get_or_insert_with_creates_entry() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .get_or_insert_with(
            GetOne::key,
            |k, _p, _s| k.len() as u64,
            |_k, v, _p, s| {
                s.result = Some(*v);
            },
        )
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState {
                result: Some(ALICE.len() as u64)
            }
        }
    );
}

#[test]
fn get_or_insert_with_does_not_call_generator_when_present() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .get_or_insert_with(
            GetOne::key,
            |_k, _p, _s| 42,
            |_k, v, _p, s| {
                s.result = Some(*v);
            },
        )
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(1) }
        }
    );
}

#[test]
fn param_get_or_insert_with_clone_value() {
    let map: TxMap<String, String> = empty_typed_map();
    let tx = map
        .prepared_tx(&GetOneParamString::SCHEMA)
        .get_or_insert(GetOneParamString::key, "default".into(), |_k, v, _p, s| {
            s.result = Some(v.clone());
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
                result: Some("default".into())
            }
        }
    );
    // executing again with a different key inserts the default there too
    assert_eq!(
        tx.execute(
            GetOneParamStringKeys { key: BOB.into() },
            GetOneParamStringParams {
                param: "hello".into()
            }
        ),
        TxResult::Completed {
            state: GetOneParamStringState {
                result: Some("default".into())
            }
        }
    );
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_cloned(&BOB.into()), Some("default".into()));
}

#[test]
fn param_get_or_insert_with() {
    let map: TxMap<String, String> = empty_typed_map();
    let tx = map
        .prepared_tx(&GetOneParamString::SCHEMA)
        .get_or_insert_with(
            GetOneParamString::key,
            |_k, p, _s| p.param.clone(),
            |_k, v, _p, s| {
                s.result = Some(v.clone());
            },
        )
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
    // existing value is not overwritten by the param-based generator
    assert_eq!(
        tx.execute(
            GetOneParamStringKeys { key: ALICE.into() },
            GetOneParamStringParams {
                param: "world".into()
            }
        ),
        TxResult::Completed {
            state: GetOneParamStringState {
                result: Some("hello".into())
            }
        }
    );
}
