//! Tests for the routing trie's split and merge protocols.

use crate::{shards::Shards, tx_map::TxMap, tx_map_builder::TxMapBuilder};

/// Routes `hash` to its leaf and splits it.
fn split(map: &TxMap<String, u64>, hash: crate::HashCode) -> bool {
    let leaf = map.custodian.route(hash).0;
    map.custodian.split_leaf(&map.indexer, leaf)
}

/// Merges the shallowest quiet branch.
fn merge(map: &TxMap<String, u64>) -> bool {
    map.custodian.merge_leaves(&map.indexer, None)
}

fn big_map() -> TxMap<String, u64> {
    TxMapBuilder::default().with_shards(Shards::_128).build()
}

fn insert_keys(map: &TxMap<String, u64>, count: u64) {
    for i in 0..count {
        map.insert(format!("key-{i}"), i);
    }
}

fn assert_entries(map: &TxMap<String, u64>, count: u64) {
    for i in 0..count {
        assert_eq!(
            map.get_copied(&format!("key-{i}")),
            Some(i),
            "missing or wrong value for key-{i}"
        );
    }
}

#[test]
fn merge_preserves_entries() {
    let map = big_map();
    insert_keys(&map, 2_000);
    let before = map.len();

    assert!(merge(&map), "expected a mergeable branch");

    assert_eq!(map.len(), before);
    assert_entries(&map, 2_000);
}

#[test]
fn split_preserves_entries() {
    let map = big_map();
    insert_keys(&map, 2_000);

    // Free seven ids by merging a branch, then split the survivor.
    assert!(merge(&map), "expected a mergeable branch");
    let before = map.len();

    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(split(&map, hash), "expected a split after a merge");

    assert_eq!(map.len(), before);
    assert_entries(&map, 2_000);
}

#[test]
fn split_respects_budget() {
    let map = big_map();
    insert_keys(&map, 64);

    assert!(merge(&map));
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(split(&map, hash));
    // Now the active leaf count is back at the maximum, so another split
    // must be refused rather than overflow the 128-leaf budget.
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(!split(&map, hash), "budget must not be exceeded");
    assert_eq!(map.len(), 64);
    assert_entries(&map, 64);
}

#[test]
fn split_then_merge_round_trips() {
    let map = big_map();
    insert_keys(&map, 1_000);

    assert!(merge(&map));
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(split(&map, hash));
    assert!(merge(&map));

    assert_eq!(map.len(), 1_000);
    assert_entries(&map, 1_000);
}
