use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_with_creates_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with(ALICE.into(), |_k, _s| 42)
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
fn insert_with_overwrites_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with(ALICE.into(), |_k, _s| 42)
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
fn param_insert_with() {
    let map: TxMap<String, String> = empty_typed_map();
    let param = "hello".to_string();
    let result = map
        .immediate_tx::<GetOneParamStringState>()
        .insert_with(ALICE.into(), move |_k, _s| param.clone())
        .get(ALICE.into(), |_k, v, s| s.result = v.cloned())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneParamStringState {
                result: Some("hello".into())
            }
        }
    );
}
