use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_default_creates_default_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(0) }));
}

#[test]
fn insert_default_overwrites_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(0) }));
}

#[test]
fn param_insert_default() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(0) }));
}
