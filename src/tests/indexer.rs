#[cfg(test)]
mod tests {
    use crate::indexer::Indexer;
    use std::hash::DefaultHasher;

    #[test]
    fn indexer_distributes_across_shards() {
        let indexer = Indexer {
            shard_count: 8,
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        let mut seen = std::collections::HashSet::new();
        for i in 0..1000u64 {
            seen.insert(indexer.index(&i));
        }
        // With 1000 keys and 8 shards, we should hit most shards
        assert!(
            seen.len() >= 4,
            "should hit at least 4 shards, got {}",
            seen.len()
        );
    }
}
