use crate::{
    result::TxResult, shards::Shards, tests::data::*, tx_map::TxMap, tx_map_builder::TxMapBuilder,
};

fn empty_map() -> TxMap<String, u64> {
    TxMapBuilder::default().with_shards(Shards::_8).build()
}

fn map_alice(alice: u64) -> TxMap<String, u64> {
    let map = empty_map();
    map.insert(ALICE.into(), alice);
    map
}

fn map_alice_bob(alice: u64, bob: u64) -> TxMap<String, u64> {
    let map = empty_map();
    map.insert(ALICE.into(), alice);
    map.insert(BOB.into(), bob);
    map
}

#[test]
fn serde_roundtrip_empty_map() {
    let map = empty_map();
    let json = serde_json::to_string(&map).unwrap();
    let deserialized: TxMap<String, u64> = serde_json::from_str(&json).unwrap();
    assert!(deserialized.is_empty());
    assert_eq!(deserialized.len(), 0);
}

#[test]
fn serde_roundtrip_single_entry() {
    let map = map_alice(42);
    let json = serde_json::to_string(&map).unwrap();
    let deserialized: TxMap<String, u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), 1);
    let val = deserialized.get_copied(&ALICE.into());
    assert_eq!(val, Some(42));
}

#[test]
fn serde_roundtrip_multiple_entries() {
    let map = map_alice_bob(10, 20);
    let json = serde_json::to_string(&map).unwrap();
    let deserialized: TxMap<String, u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized.get_copied(&ALICE.into()), Some(10));
    assert_eq!(deserialized.get_copied(&BOB.into()), Some(20));
}

#[test]
fn serde_roundtrip_with_integer_keys() {
    let map: TxMap<u64, String> = TxMapBuilder::default().with_shards(Shards::_8).build();
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    let json = serde_json::to_string(&map).unwrap();
    let deserialized: TxMap<u64, String> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized.get_cloned(&1), Some("one".to_string()));
    assert_eq!(deserialized.get_cloned(&2), Some("two".to_string()));
}

#[test]
fn serde_roundtrip_preserves_all_values() {
    let map = empty_map();
    let entries: Vec<(String, u64)> = vec![
        (ALICE.into(), 100u64),
        (BOB.into(), 200u64),
        (CHUCK.into(), 300u64),
        (DAVE.into(), 400u64),
    ];
    for (k, v) in &entries {
        map.insert(k.clone(), *v);
    }
    let json = serde_json::to_string(&map).unwrap();
    let deserialized: TxMap<String, u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), entries.len());
    for (k, v) in &entries {
        assert_eq!(deserialized.get_copied(k), Some(*v));
    }
}

#[test]
fn serde_roundtrip_tx_result_completed() {
    let result: TxResult<u64> = TxResult::Completed(42);
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TxResult<u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(result, deserialized);
}

#[test]
fn serde_roundtrip_tx_result_requirement_not_met() {
    let result: TxResult<u64> = TxResult::RequirementNotMet(0, "balance_check".to_string(), 99);
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TxResult<u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(result, deserialized);
}

#[test]
fn serde_roundtrip_tx_result_with_string() {
    let result: TxResult<String> = TxResult::Completed("hello".to_string());
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TxResult<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(result, deserialized);

    let result: TxResult<String> =
        TxResult::RequirementNotMet(2, "check".to_string(), "state".to_string());
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TxResult<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(result, deserialized);
}
