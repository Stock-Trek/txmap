use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_with_if_absent_creates_entry() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with_if_absent(GetOne::key, |_k, _p, _s| 42)
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
fn insert_with_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with_if_absent(GetOne::key, |_k, _p, _s| 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(1) }
        }
    );
}
