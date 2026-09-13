//! Tests for the routing trie's split and merge protocols.

use crate::{shards::Shards, tx_map::TxMap, tx_map_builder::TxMapBuilder};

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

    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.merge_leaves(hash), "expected a mergeable branch");

    assert_eq!(map.len(), before);
    assert_entries(&map, 2_000);
}

#[test]
fn split_preserves_entries() {
    let map = big_map();
    insert_keys(&map, 2_000);

    // Free seven ids by merging a branch, then split the survivor.
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.merge_leaves(hash), "expected a mergeable branch");
    let before = map.len();

    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.split_leaf(hash), "expected a split after a merge");

    assert_eq!(map.len(), before);
    assert_entries(&map, 2_000);
}

#[test]
fn split_respects_budget() {
    let map = big_map();
    insert_keys(&map, 64);

    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.merge_leaves(hash));
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.split_leaf(hash));
    // Now the active leaf count is back at the maximum, so another split
    // must be refused rather than overflow the 128-leaf budget.
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(!map.split_leaf(hash), "budget must not be exceeded");
    assert_eq!(map.len(), 64);
    assert_entries(&map, 64);
}

#[test]
fn split_then_merge_round_trips() {
    let map = big_map();
    insert_keys(&map, 1_000);

    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.merge_leaves(hash));
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.split_leaf(hash));
    let hash = map.indexer.hash(&"key-0".to_string());
    assert!(map.merge_leaves(hash));

    assert_eq!(map.len(), 1_000);
    assert_entries(&map, 1_000);
}
