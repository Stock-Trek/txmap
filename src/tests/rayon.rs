use crate::{
    lock_policies::rwlock_policy::RwLockPolicy,
    tests::{creators::*, data::*},
    tx_map::TxMap,
    tx_map_builder::TxMapBuilder,
};
use rayon::prelude::*;

#[test]
fn par_iter_empty_map() {
    let map = empty_map();
    let count = map.par_iter().count();
    assert_eq!(count, 0);
}

#[test]
fn par_iter_single_entry() {
    let map = map_alice(42);
    let mut entries: Vec<(String, u64)> = map.par_iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort();
    assert_eq!(entries, vec![(ALICE.into(), 42)]);
}

#[test]
fn par_iter_multiple_entries() {
    let map = map_alice_bob_chuck_dave(10, 20, 30, 40);
    let mut entries: Vec<(String, u64)> = map.par_iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            (ALICE.into(), 10),
            (BOB.into(), 20),
            (CHUCK.into(), 30),
            (DAVE.into(), 40),
        ]
    );
}

#[test]
fn par_iter_matches_len() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    let count = map.par_iter().count();
    assert_eq!(count, map.len());
}

#[test]
fn par_iter_reduces_all_entries() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    let sum: u64 = map.par_iter().map(|(_, v)| v).sum();
    assert_eq!(sum, 10);
}

#[test]
fn par_iter_via_into_par_iter() {
    let map = map_alice_bob(100, 200);
    let mut entries: Vec<(String, u64)> = (&map)
        .into_par_iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    entries.sort();
    assert_eq!(entries, vec![(ALICE.into(), 100), (BOB.into(), 200),]);
}

#[test]
fn par_iter_via_mut_ref() {
    let mut map = map_alice_bob(5, 10);
    let sum: u64 = (&mut map).into_par_iter().map(|(_, v)| *v).sum();
    assert_eq!(sum, 15);
}

#[test]
fn par_keys_collects_all_keys() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    let mut keys: Vec<String> = map.par_keys().map(|k| k.clone()).collect();
    keys.sort();
    let expected: Vec<String> = vec![ALICE.into(), BOB.into(), CHUCK.into(), DAVE.into()];
    assert_eq!(keys, expected);
}

#[test]
fn par_values_collects_all_values() {
    let map = map_alice_bob_chuck_dave(10, 20, 30, 40);
    let mut values: Vec<u64> = map.par_values().copied().collect();
    values.sort();
    assert_eq!(values, vec![10, 20, 30, 40]);
}

#[test]
fn into_par_iter_consumes_map() {
    let map = map_alice_bob(100, 200);
    let mut entries: Vec<(String, u64)> = map.into_par_iter().collect();
    entries.sort();
    assert_eq!(entries, vec![(ALICE.into(), 100), (BOB.into(), 200),]);
}

#[test]
fn par_iter_with_rwlock_policy() {
    let map: TxMap<String, u64, RwLockPolicy> = TxMapBuilder::default()
        .with_lock_policy::<RwLockPolicy>()
        .build();
    map.insert(ALICE.into(), 1);
    map.insert(BOB.into(), 2);
    let sum: u64 = map.par_iter().map(|(_, v)| *v).sum();
    assert_eq!(sum, 3);
}
