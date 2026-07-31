use crate::{
    prelude::TxMap,
    tests::{creators::*, data::*},
};
use std::collections::HashMap as StdHashMap;

#[test]
fn contains_key_returns_present_and_absent() {
    let map = map_alice_bob(1, 2);
    assert!(map.contains_key(&ALICE.into()));
    assert!(map.contains_key(&BOB.into()));
    assert!(!map.contains_key(&CHUCK.into()));
    assert!(!map.contains_key(&DAVE.into()));
}

#[test]
fn remove_entry_returns_key_and_value() {
    let map = map_alice_bob(1, 2);
    assert_eq!(
        map.remove_entry(&ALICE.into()),
        Some((ALICE.to_string(), 1))
    );
    assert_eq!(map.remove_entry(&ALICE.into()), None);
    assert_eq!(map.len(), 1);
}

#[test]
fn keys_iterates_all_keys() {
    let map = map_alice_bob_chuck(1, 2, 3);
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        vec![&ALICE.to_string(), &BOB.to_string(), &CHUCK.to_string()]
    );
}

#[test]
fn values_iterates_all_values() {
    let map = map_alice_bob_chuck(1, 2, 3);
    let mut values: Vec<&u64> = map.values().collect();
    values.sort();
    assert_eq!(values, vec![&1, &2, &3]);
}

#[test]
fn capacity_is_at_least_len() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    assert!(map.capacity() >= map.len());
}

#[test]
fn reserve_increases_capacity() {
    let map = empty_map();
    assert!(map.capacity() < 1000);
    map.reserve(1000);
    assert!(map.capacity() >= 1000);
}

#[test]
fn try_reserve_succeeds() {
    let map = empty_map();
    assert!(map.try_reserve(1000).is_ok());
    assert!(map.capacity() >= 1000);
}

#[test]
fn shrink_to_fit_keeps_entries() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    map.reserve(1000);
    map.shrink_to_fit();
    assert_eq!(map.len(), 4);
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

#[test]
fn shrink_to_keeps_entries() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    map.reserve(1000);
    map.shrink_to(10);
    assert_eq!(map.len(), 4);
    assert!(map.capacity() >= 4);
}

#[test]
fn hasher_returns_builder() {
    let map = empty_map();
    let _ = map.hasher();
}

#[test]
fn drain_removes_all_entries() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    let drained: StdHashMap<String, u64> = map.drain().collect();
    assert_eq!(drained.len(), 4);
    assert!(map.is_empty());
}

#[test]
fn drain_dropped_midway_clears_map() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    {
        let mut drain = map.drain();
        let _first = drain.next();
    }
    assert!(map.is_empty());
}

#[test]
fn into_iter_consumes_map() {
    let map = map_alice_bob(1, 2);
    let collected: StdHashMap<String, u64> = map.into_iter().collect();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected.get(ALICE), Some(&1));
    assert_eq!(collected.get(BOB), Some(&2));
}

#[test]
fn into_keys_consumes_map() {
    let map = map_alice_bob(1, 2);
    let mut keys: Vec<String> = map.into_keys().collect();
    keys.sort();
    assert_eq!(keys, vec![ALICE.to_string(), BOB.to_string()]);
}

#[test]
fn into_values_consumes_map() {
    let map = map_alice_bob(1, 2);
    let mut values: Vec<u64> = map.into_values().collect();
    values.sort();
    assert_eq!(values, vec![1, 2]);
}

#[test]
fn from_iterator_collects_entries() {
    let map: TxMap<String, u64> = vec![(ALICE.into(), 1), (BOB.into(), 2)]
        .into_iter()
        .collect();
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
    assert_eq!(map.get_copied(&BOB.into()), Some(2));
}

#[test]
fn extend_adds_owned_entries() {
    let map = map_alice(1);
    let mut map = map;
    map.extend(vec![(BOB.into(), 2), (CHUCK.into(), 3)]);
    assert_eq!(map.len(), 3);
    assert_eq!(map.get_copied(&CHUCK.into()), Some(3));
}

#[test]
fn extend_adds_reference_entries() {
    let map = map_alice(1);
    let mut map = map;
    let extra = [(BOB.to_string(), 2), (CHUCK.to_string(), 3)];
    map.extend(extra.iter().map(|(k, v)| (k, v)));
    assert_eq!(map.len(), 3);
    assert_eq!(map.get_copied(&BOB.into()), Some(2));
}

#[test]
fn from_array_builds_map() {
    let map: TxMap<String, u64> = TxMap::from([(ALICE.into(), 1), (BOB.into(), 2)]);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

#[test]
fn partial_eq_compares_entries() {
    let map1 = map_alice_bob_chuck(1, 2, 3);
    let map2 = map_alice_bob_chuck(1, 2, 3);
    let map3 = map_alice_bob_chuck(1, 2, 4);
    let map4 = map_alice_bob(1, 2);
    assert_eq!(map1, map2);
    assert_ne!(map1, map3);
    assert_ne!(map1, map4);
}

#[test]
fn debug_formats_entries() {
    let map = map_alice(7);
    let debug = format!("{map:?}");
    assert!(debug.contains(ALICE));
    assert!(debug.contains("7"));
}

#[test]
fn into_iter_for_references() {
    let map = map_alice(7);
    let by_ref: StdHashMap<&String, &u64> = (&map).into_iter().collect();
    assert_eq!(by_ref.get(&ALICE.to_string()), Some(&&7));

    let mut map = map;
    let by_mut_ref: StdHashMap<&String, &u64> = (&mut map).into_iter().collect();
    assert_eq!(by_mut_ref.get(&ALICE.to_string()), Some(&&7));
}
