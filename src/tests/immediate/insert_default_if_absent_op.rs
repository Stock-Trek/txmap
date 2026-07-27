use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_default_if_absent_creates_default_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default_if_absent(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(0) }));
}

#[test]
fn insert_default_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default_if_absent(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(1) }));
}
