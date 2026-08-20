use crate::{
    indexer::Indexer, key::TxKey, lock_policies::lock_policy::LockPolicy,
    result::MISSING_LOCK_GUARD_ERROR, shard::Shard, shard_ops::ShardOps,
};
use std::hash::{BuildHasher, Hash};

pub(crate) struct MultiShardOps;

impl MultiShardOps {
    #[inline]
    pub fn move_value<K, V, L, S>(
        write_guards: &mut [Option<L::WriteGuard<'_, Shard<K, V>>>],
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
        indexer: &Indexer<S>,
    ) where
        K: Clone + Hash + PartialEq,
        L: LockPolicy,
        S: BuildHasher,
    {
        let removed = {
            let shard = Self::shard::<K, V, L>(write_guards, key_from);
            ShardOps::remove_entry::<K, V>(shard, key_from.hash_code, &key_from.key)
        };
        let shard_to = write_guards[key_to.shard_index.0 as usize]
            .as_mut()
            .expect(MISSING_LOCK_GUARD_ERROR);
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
    pub fn swap_value<K, V, L, S>(
        write_guards: &mut [Option<L::WriteGuard<'_, Shard<K, V>>>],
        key_a: &TxKey<K>,
        key_b: &TxKey<K>,
        indexer: &Indexer<S>,
    ) where
        K: Clone + Hash + PartialEq,
        L: LockPolicy,
        S: BuildHasher,
    {
        let a = {
            let shard = Self::shard::<K, V, L>(write_guards, key_a);
            ShardOps::remove_entry::<K, V>(shard, key_a.hash_code, &key_a.key)
        };
        let b = {
            let shard = Self::shard::<K, V, L>(write_guards, key_b);
            ShardOps::remove_entry::<K, V>(shard, key_b.hash_code, &key_b.key)
        };
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    {
                        let shard = Self::shard::<K, V, L>(write_guards, key_a);
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
                        let shard = Self::shard::<K, V, L>(write_guards, key_b);
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
                    let shard = Self::shard::<K, V, L>(write_guards, key_b);
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
                        let shard = Self::shard::<K, V, L>(write_guards, key_a);
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

    #[inline]
    fn shard<'ex, K, V, L>(
        write_guards: &'ex mut [Option<L::WriteGuard<'_, Shard<K, V>>>],
        key: &TxKey<K>,
    ) -> &'ex mut Shard<K, V>
    where
        L: LockPolicy,
    {
        write_guards[key.shard_index.0 as usize]
            .as_mut()
            .expect(MISSING_LOCK_GUARD_ERROR)
    }
}
