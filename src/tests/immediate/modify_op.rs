use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn modify_existing_key() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += 5;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(6) }));
}

#[test]
fn modify_missing_key_is_noop() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v = 42;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
}
