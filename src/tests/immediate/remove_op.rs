use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn remove() {
    let map = map_alice(42);
    let result = map
        .immediate_tx::<GetOneState>()
        .remove(ALICE.into(), |entry, s| {
            s.result = entry.map(|(_, v)| v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(42) })
    );
    assert!(map.is_empty());
}

#[test]
fn remove_missing_key() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .remove(ALICE.into(), |entry, s| {
            s.result = entry.map(|(_, v)| v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
}
