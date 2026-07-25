use crate::prelude::*;

#[test]
fn shard_indexes() {
    assert_eq!(ShardCount::from(Shards::_8).0, 8);
    assert_eq!(ShardCount::from(Shards::_16).0, 16);
    assert_eq!(ShardCount::from(Shards::_32).0, 32);
    assert_eq!(ShardCount::from(Shards::_64).0, 64);
    assert_eq!(ShardCount::from(Shards::_128).0, 128);
}

#[test]
fn bitmasks() {
    assert_eq!(shards_to_bitmask(Shards::_8).0, 1 << 7);
    assert_eq!(shards_to_bitmask(Shards::_16).0, 1 << 15);
    assert_eq!(shards_to_bitmask(Shards::_32).0, 1 << 31);
    assert_eq!(shards_to_bitmask(Shards::_64).0, 1 << 63);
    assert_eq!(shards_to_bitmask(Shards::_128).0, 1 << 127);
}

fn shards_to_bitmask(shards: Shards) -> BitMask {
    let shard_count = ShardCount::from(shards);
    let max_shard_index = ShardIndex(shard_count.0 - 1);
    max_shard_index.bitmask()
}

#[test]
fn sanity_check() {
    for shards in [
        Shards::_8,
        Shards::_16,
        Shards::_32,
        Shards::_64,
        Shards::_128,
    ] {
        let map: TxMap<u64, u64> = TxMap::new(shards);
        assert!(map.is_empty());
        map.insert(1, 10);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&1, |v| *v), Some(10));
    }
}
