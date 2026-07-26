use crate::prelude::*;
use hashbrown::HashSet;

#[test]
fn shard_count_hash_distributes_across_shards() {
    for shards in [
        Shards::_8,
        Shards::_16,
        Shards::_32,
        Shards::_64,
        Shards::_128,
    ] {
        let shard_count = ShardCount::from(shards);
        let mut seen = HashSet::new();
        for i in 0u8..128 {
            let index = ShardIndex(i).bitmask();
            seen.insert(index);
        }
        // With all possible shard indices, we should hit all shards
        assert!(
            seen.len() >= shard_count.0 as usize,
            "should hit all shards, got {}, expected at least {}",
            seen.len(),
            shard_count.0
        );
    }
}
