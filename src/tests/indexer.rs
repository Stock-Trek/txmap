#[cfg(test)]
mod tests {
    use crate::shard_count::ShardCount;

    #[test]
    fn shard_count_hash_distributes_across_shards() {
        for sc in ShardCount::all() {
            let shard_count = u8::from(sc);
            let mut seen = std::collections::HashSet::new();
            for i in 0..10_000 {
                seen.insert(ShardCount::shard_index(shard_count, &i));
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
