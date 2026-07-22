#[cfg(test)]
mod tests {
    use crate::{new_types::HashCode, shard_count::ShardCount};

    #[test]
    fn shard_count_hash_distributes_across_shards() {
        for sc in vec![
            ShardCount::_8,
            ShardCount::_16,
            ShardCount::_32,
            ShardCount::_64,
            ShardCount::_128,
        ] {
            let shard_count = u8::from(sc);
            let mut seen = std::collections::HashSet::new();
            for i in 0..10_000 {
                seen.insert(ShardCount::shard_index(shard_count, HashCode(i)));
            }
            // With 10_000 keys, we should hit all shards
            assert!(
                seen.len() == shard_count as usize,
                "should hit all shards, got {}",
                seen.len()
            );
        }
    }
}
