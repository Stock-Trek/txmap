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
