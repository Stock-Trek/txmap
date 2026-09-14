use crate::{
    indexer::Indexer, key::TxKey, lock_guard::LockGuard, multi_shard_ops::MultiShardOps,
    new_types::BitMask, shard_ops::ShardOps,
};
use std::hash::{BuildHasher, Hash};

#[allow(clippy::type_complexity)]
pub(crate) enum ImmediateOp<'tx, K, V, STATE> {
    Get {
        key: TxKey<K>,
        get: Box<dyn FnOnce(&K, Option<&V>, &mut STATE) + 'tx>,
    },
    GetOrInsert {
        key: TxKey<K>,
        value: V,
        get: Box<dyn FnOnce(&K, &V, &mut STATE) + 'tx>,
    },
    GetOrInsertWith {
        key: TxKey<K>,
        value_generator: Box<dyn FnOnce(&K, &mut STATE) -> V + 'tx>,
        get: Box<dyn FnOnce(&K, &V, &mut STATE) + 'tx>,
    },
    InsertWith {
        key: TxKey<K>,
        value_generator: Box<dyn FnOnce(&K, &mut STATE) -> V + 'tx>,
    },
    InsertWithIfAbsent {
        key: TxKey<K>,
        value_generator: Box<dyn FnOnce(&K, &mut STATE) -> V + 'tx>,
    },
    Modify {
        key: TxKey<K>,
        mutate: Box<dyn FnOnce(&K, &mut V, &mut STATE) + 'tx>,
    },
    MoveValue {
        key_from: TxKey<K>,
        key_to: TxKey<K>,
    },
    Remove {
        key: TxKey<K>,
    },
    RemoveIf {
        key: TxKey<K>,
        condition: Box<dyn FnOnce(&K, &V, &mut STATE) -> bool + 'tx>,
    },
    SwapValue {
        key_a: TxKey<K>,
        key_b: TxKey<K>,
    },
    Update {
        key: TxKey<K>,
        transform: Box<dyn FnOnce(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx>,
    },
}

impl<'tx, K, V, STATE> ImmediateOp<'tx, K, V, STATE> {
    pub fn bitmask(&self) -> BitMask {
        match self {
            Self::Get { key, .. }
            | Self::GetOrInsert { key, .. }
            | Self::GetOrInsertWith { key, .. }
            | Self::InsertWith { key, .. }
            | Self::InsertWithIfAbsent { key, .. }
            | Self::Modify { key, .. }
            | Self::Remove { key, .. }
            | Self::RemoveIf { key, .. }
            | Self::Update { key, .. } => key.shard_index.bitmask(),
            Self::MoveValue {
                key_from, key_to, ..
            } => key_from.shard_index.bitmask() | key_to.shard_index.bitmask(),
            Self::SwapValue { key_a, key_b, .. } => {
                key_a.shard_index.bitmask() | key_b.shard_index.bitmask()
            }
        }
    }
}

impl<'tx, K, V, STATE> ImmediateOp<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq,
{
    pub fn apply<S>(
        self,
        lock_guard: &mut LockGuard<'_, K, V>,
        indexer: &Indexer<S>,
        state: &mut STATE,
    ) where
        S: BuildHasher,
    {
        match self {
            Self::Get { key, get } => {
                let shard = lock_guard.shard_for_key(&key);
                let value_ref = ShardOps::value_ref(shard, key.hash_code, &key.key);
                (get)(&key.key, value_ref, state)
            }
            Self::GetOrInsert { key, value, get } => {
                let shard = lock_guard.shard_for_key(&key);
                let value_ref = ShardOps::get_or_insert::<K, V, S>(
                    shard,
                    key.hash_code,
                    &key.key,
                    value,
                    indexer,
                );
                (get)(&key.key, value_ref, state)
            }
            Self::GetOrInsertWith {
                key,
                value_generator,
                get,
            } => {
                let shard = lock_guard.shard_for_key(&key);
                let value_ref = ShardOps::get_or_insert_with(
                    shard,
                    key.hash_code,
                    &key.key,
                    |k| (value_generator)(k, state),
                    indexer,
                );
                (get)(&key.key, value_ref, state)
            }
            Self::InsertWith {
                key,
                value_generator,
            } => {
                let new_value = (value_generator)(&key.key, state);
                let shard = lock_guard.shard_for_key(&key);
                ShardOps::insert::<K, V, S>(shard, key.hash_code, key.key, new_value, indexer);
            }
            Self::InsertWithIfAbsent {
                key,
                value_generator,
            } => {
                let shard = lock_guard.shard_for_key(&key);
                ShardOps::insert_if_absent::<K, V, S>(
                    shard,
                    key.hash_code,
                    key.key,
                    |k| (value_generator)(k, state),
                    indexer,
                );
            }
            Self::Modify { key, mutate } => {
                let shard = lock_guard.shard_for_key(&key);
                ShardOps::modify(shard, key.hash_code, &key.key, |k, v| mutate(k, v, state));
            }
            Self::MoveValue { key_from, key_to } => {
                MultiShardOps::move_value::<K, V, S>(lock_guard, &key_from, &key_to, indexer);
            }
            Self::Remove { key } => {
                let shard = lock_guard.shard_for_key(&key);
                ShardOps::remove_entry::<K, V>(shard, key.hash_code, &key.key);
            }
            Self::RemoveIf { key, condition } => {
                let shard = lock_guard.shard_for_key(&key);
                ShardOps::remove_if(shard, key.hash_code, &key.key, |k, v| {
                    condition(k, v, state)
                });
            }
            Self::SwapValue { key_a, key_b } => {
                MultiShardOps::swap_value::<K, V, S>(lock_guard, &key_a, &key_b, indexer);
            }
            Self::Update { key, transform } => {
                let shard = lock_guard.shard_for_key(&key);
                ShardOps::update(
                    shard,
                    key.hash_code,
                    key.key,
                    |k, v_opt| transform(k, v_opt, state),
                    indexer,
                );
            }
        }
    }
}
