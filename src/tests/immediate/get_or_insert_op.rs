use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn get_or_insert_creates_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .get_or_insert(ALICE.into(), 42, |_k, v, s| s.result = Some(*v))
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneState { result: Some(42) }
        }
    );
    assert_eq!(map.get_copied(&ALICE.into()), Some(42));
}

#[test]
fn get_or_insert_does_not_overwrite_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .get_or_insert(ALICE.into(), 42, |_k, v, s| s.result = Some(*v))
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneState { result: Some(1) }
        }
    );
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

#[test]
fn get_or_insert_with_creates_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .get_or_insert_with(
            ALICE.into(),
            |k, _s| k.len() as u64,
            |_k, v, s| s.result = Some(*v),
        )
        .execute();
    assert_eq!(
        result,
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
    let mut generator_calls = 0;
    let result = map
        .immediate_tx::<GetOneState>()
        .get_or_insert_with(
            ALICE.into(),
            |_k, _s| {
                generator_calls += 1;
                42
            },
            |_k, v, s| s.result = Some(*v),
        )
        .execute();
    assert_eq!(generator_calls, 0);
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneState { result: Some(1) }
        }
    );
}
