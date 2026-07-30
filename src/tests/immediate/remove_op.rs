use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn remove() {
    let map = map_alice(42);
    let result = map
        .immediate_tx::<GetOneState>()
        .remove(ALICE.into())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
    assert!(map.is_empty());
}

#[test]
fn remove_missing_key() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .remove(ALICE.into())
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
}
