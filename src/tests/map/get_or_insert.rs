use crate::tests::{creators::*, data::*};

#[test]
fn get_with_or_insert_inserts_when_absent() {
    let map = empty_map();
    let returned = map.get_with_or_insert(&ALICE.into(), |v| *v, 42);
    assert_eq!(returned, 42);
    assert_eq!(map.get_copied(&ALICE.into()), Some(42));
    assert_eq!(map.len(), 1);
}

#[test]
fn get_with_or_insert_returns_existing_value() {
    let map = map_alice(1);
    let returned = map.get_with_or_insert(&ALICE.into(), |v| *v, 42);
    assert_eq!(returned, 1);
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
    assert_eq!(map.len(), 1);
}

#[test]
fn get_with_or_insert_with_inserts_when_absent() {
    let map = empty_map();
    let returned = map.get_with_or_insert_with(&ALICE.into(), |v| *v, |key| key.len() as u64);
    assert_eq!(returned, ALICE.len() as u64);
    assert_eq!(map.get_copied(&ALICE.into()), Some(ALICE.len() as u64));
}

#[test]
fn get_with_or_insert_with_returns_existing_value() {
    let map = map_alice(1);
    let returned = map.get_with_or_insert_with(&ALICE.into(), |v| *v, |_key| 42);
    assert_eq!(returned, 1);
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

#[test]
fn get_with_or_insert_with_cloned_value_type() {
    let map: crate::prelude::TxMap<String, String> = empty_typed_map();
    let returned = map.get_with_or_insert(&ALICE.into(), |v| v.clone(), "default".into());
    assert_eq!(returned, "default");
    let returned = map.get_with_or_insert(&ALICE.into(), |v| v.clone(), "other".into());
    assert_eq!(returned, "default");
    assert_eq!(map.get_cloned(&ALICE.into()), Some("default".into()));
}
