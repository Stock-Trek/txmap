use crate::{
    key::TxKey, lock_policies::lock_policy::LockPolicy, result::MISSING_LOCK_GUARD_ERROR,
    shard::Shard, shard_ops::ShardOps,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct MultiShardOps;

impl MultiShardOps {
    #[inline]
    pub fn move_value<K, V, L>(
        write_guards: &mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>,
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let removed = {
            let shard = Self::shard::<K, V, L>(write_guards, key_from);
            ShardOps::remove_entry::<K, V>(shard, key_from)
        };
        let shard_to = write_guards
            .get_mut(key_to.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        if let Some(entry) = removed {
            ShardOps::insert(shard_to, key_to, entry.1);
        } else {
            ShardOps::remove_entry(shard_to, key_to);
        }
    }
    #[inline]
    pub fn swap_value<K, V, L>(
        write_guards: &mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>,
        key_a: &TxKey<K>,
        key_b: &TxKey<K>,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let a = {
            let shard = Self::shard::<K, V, L>(write_guards, key_a);
            ShardOps::remove_entry::<K, V>(shard, key_a)
        };
        let b = {
            let shard = Self::shard::<K, V, L>(write_guards, key_b);
            ShardOps::remove_entry::<K, V>(shard, key_b)
        };
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    {
                        let shard = Self::shard::<K, V, L>(write_guards, key_a);
                        ShardOps::insert_with_duplicate_key(shard, key_a, a_key, b_value);
                    }
                    {
                        let shard = Self::shard::<K, V, L>(write_guards, key_b);
                        ShardOps::insert_with_duplicate_key(shard, key_b, b_key, a_value);
                    }
                }
                None => {
                    let shard = Self::shard::<K, V, L>(write_guards, key_b);
                    ShardOps::insert(shard, key_b, a_value);
                }
            },
            None => {
                if let Some((_, b_value)) = b {
                    {
                        let shard = Self::shard::<K, V, L>(write_guards, key_a);
                        ShardOps::insert(shard, key_a, b_value);
                    }
                }
            }
        }
    }
    #[inline]
    fn shard<'ex, K, V, L>(
        write_guards: &'ex mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>,
        key: &TxKey<K>,
    ) -> &'ex mut Shard<K, V>
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        write_guards
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
    }
}
