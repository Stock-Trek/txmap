use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_with_if_absent_creates_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with_if_absent(ALICE.into(), |_k, _s| 42)
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneState { result: Some(42) }
        }
    );
}

#[test]
fn insert_with_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with_if_absent(ALICE.into(), |_k, _s| 42)
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneState { result: Some(1) }
        }
    );
}
