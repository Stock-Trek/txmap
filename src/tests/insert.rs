use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn insert_with_creates_entry() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn insert_with_overwrites_existing() {
    let map = map_alice(1);
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn insert_with_if_absent_creates_entry() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_with_if_absent(GetOne::key, |_k, _p, _s| 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn insert_with_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_with_if_absent(GetOne::key, |_k, _p, _s| 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(1) })
    );
}

#[test]
fn insert_default_creates_default_entry() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_default(GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn insert_default_overwrites_existing() {
    let map = map_alice(1);
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_default(GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn insert_default_if_absent_creates_default_entry() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_default_if_absent(GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn insert_default_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let tx = map
        .prepare_transaction(&GetOne::SCHEMA)
        .insert_default_if_absent(GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed(GetOneState { result: Some(1) })
    );
}

#[test]
fn duplicate_keys_dont_cause_issues() {
    let map = empty_map();
    for i in 0..=100 {
        map.insert(ALICE.into(), i);
    }
    assert_eq!(map.len(), 1);
    assert_eq!(map.get_copied(&ALICE.into()), Some(100));
}

#[test]
fn huge_string_keys() {
    let map = empty_map();
    let big_key = "x".repeat(10_000);
    map.insert(big_key.clone(), 42);
    assert_eq!(map.get_with(&big_key, |v| *v), Some(42));
}

#[test]
fn clear_then_reinsert() {
    let map = empty_map();
    map.insert(ALICE.into(), 1);
    map.clear();
    assert!(map.is_empty());
    map.insert(ALICE.into(), 2);
    assert_eq!(map.get_with(&ALICE.into(), |v| *v), Some(2));
}

#[test]
fn large_number_of_keys() {
    let map = empty_typed_map();
    for i in 0..10_000 {
        map.insert(i, i * 3);
    }
    assert_eq!(map.len(), 10_000);
    // Verify a few values
    for i in (0..10_000).step_by(1000) {
        assert_eq!(map.get_with(&i, |v| *v), Some(i * 3));
    }
}
