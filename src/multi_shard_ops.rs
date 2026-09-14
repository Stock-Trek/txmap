use crate::{indexer::Indexer, key::TxKey, lock_guards::LockGuard, shard_ops::ShardOps};
use std::hash::{BuildHasher, Hash};

pub(crate) struct MultiShardOps;

impl MultiShardOps {
    #[inline]
    pub fn move_value<K, V, S>(
        lock_guard: &mut LockGuard<'_, K, V>,
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
        indexer: &Indexer<S>,
    ) where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let removed = {
            let shard = lock_guard.shard_for_key(key_from);
            ShardOps::remove_entry::<K, V>(shard, key_from.hash_code, &key_from.key)
        };
        let shard_to = lock_guard.shard_for_key(key_to);
        if let Some(entry) = removed {
            ShardOps::insert::<K, V, S>(
                shard_to,
                key_to.hash_code,
                key_to.key.clone(),
                entry.1,
                indexer,
            );
        } else {
            ShardOps::remove_entry(shard_to, key_to.hash_code, &key_to.key);
        }
    }

    #[inline]
    pub fn swap_value<K, V, S>(
        lock_guard: &mut LockGuard<'_, K, V>,
        key_a: &TxKey<K>,
        key_b: &TxKey<K>,
        indexer: &Indexer<S>,
    ) where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let a = {
            let shard = lock_guard.shard_for_key(key_a);
            ShardOps::remove_entry::<K, V>(shard, key_a.hash_code, &key_a.key)
        };
        let b = {
            let shard = lock_guard.shard_for_key(key_b);
            ShardOps::remove_entry::<K, V>(shard, key_b.hash_code, &key_b.key)
        };
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    {
                        let shard = lock_guard.shard_for_key(key_a);
                        ShardOps::insert_with_duplicate_key(
                            shard,
                            key_a.hash_code,
                            &key_a.key,
                            a_key,
                            b_value,
                            indexer,
                        );
                    }
                    {
                        let shard = lock_guard.shard_for_key(key_b);
                        ShardOps::insert_with_duplicate_key(
                            shard,
                            key_b.hash_code,
                            &key_b.key,
                            b_key,
                            a_value,
                            indexer,
                        );
                    }
                }
                None => {
                    let shard = lock_guard.shard_for_key(key_b);
                    ShardOps::insert::<K, V, S>(
                        shard,
                        key_b.hash_code,
                        key_b.key.clone(),
                        a_value,
                        indexer,
                    );
                }
            },
            None => {
                if let Some((_, b_value)) = b {
                    {
                        let shard = lock_guard.shard_for_key(key_a);
                        ShardOps::insert::<K, V, S>(
                            shard,
                            key_a.hash_code,
                            key_a.key.clone(),
                            b_value,
                            indexer,
                        );
                    }
                }
            }
        }
    }
}
