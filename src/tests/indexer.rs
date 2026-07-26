use crate::prelude::*;
use hashbrown::HashSet;

#[test]
fn shard_count_hash_distributes_across_shards() {
    for shards in vec![
        Shards::_8,
        Shards::_16,
        Shards::_32,
        Shards::_64,
        Shards::_128,
    ] {
        let shard_count = ShardCount::from(shards);
        let mut seen = HashSet::new();
        for i in 0..10_000 {
            let index = ShardIndex(i).bitmask();
            seen.insert(index);
        }
        // With 10_000 keys, we should hit all shards
        assert!(
            seen.len() == shard_count.0 as usize,
            "should hit all shards, got {}",
            seen.len()
        );
    }
}
