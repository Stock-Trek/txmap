use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    multi_shard_ops::MultiShardOps, new_types::BitMask, shard_ops::ShardOps,
};
use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[allow(clippy::type_complexity)]
pub(crate) enum ImmediateOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    Get {
        key: TxKey<K>,
        get: Box<dyn Fn(&K, Option<&V>, &mut STATE) + 'tx>,
    },
    InsertWith {
        key: TxKey<K>,
        value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
    },
    InsertWithIfAbsent {
        key: TxKey<K>,
        value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
    },
    Modify {
        key: TxKey<K>,
        mutate: Box<dyn Fn(&K, &mut V, &mut STATE) + 'tx>,
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
        condition: Box<dyn Fn(&K, &V, &mut STATE) -> bool + 'tx>,
    },
    SwapValue {
        key_a: TxKey<K>,
        key_b: TxKey<K>,
    },
    Update {
        key: TxKey<K>,
        transform: Box<dyn Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx>,
    },
}

impl<'tx, K, V, STATE> ImmediateOp<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq,
{
    pub fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        match self {
            Self::Get { key, .. } => (key.shard_index.bitmask(), BitMask::ZERO),
            Self::InsertWith { key, .. }
            | Self::InsertWithIfAbsent { key, .. }
            | Self::Modify { key, .. }
            | Self::Remove { key, .. }
            | Self::RemoveIf { key, .. }
            | Self::Update { key, .. } => (BitMask::ZERO, key.shard_index.bitmask()),
            Self::MoveValue {
                key_from, key_to, ..
            } => (
                BitMask::ZERO,
                key_from.shard_index.bitmask() | key_to.shard_index.bitmask(),
            ),
            Self::SwapValue { key_a, key_b, .. } => (
                BitMask::ZERO,
                key_a.shard_index.bitmask() | key_b.shard_index.bitmask(),
            ),
        }
    }
    pub fn apply<L>(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE)
    where
        L: LockPolicy,
    {
        match self {
            Self::Get { key, get } => {
                let shard =
                    if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                        lock_guards.write_guard(key).deref_mut()
                    } else {
                        lock_guards.read_guard(key).deref()
                    };
                let value_ref = ShardOps::value_ref(shard, key);
                (get)(&key.key, value_ref, state)
            }
            Self::InsertWith {
                key,
                value_generator,
            } => {
                let new_value = (value_generator)(&key.key, state);
                let write_guard = lock_guards.write_guard(key);
                ShardOps::insert::<K, V>(write_guard, key, new_value);
            }
            Self::InsertWithIfAbsent {
                key,
                value_generator,
            } => {
                let write_guard = lock_guards.write_guard(key);
                ShardOps::insert_if_absent::<K, V>(write_guard, key, || {
                    (value_generator)(&key.key, state)
                });
            }
            Self::Modify { key, mutate } => {
                let shard = lock_guards.write_guard(key);
                ShardOps::modify(shard, key, |k, v| mutate(k, v, state));
            }
            Self::MoveValue { key_from, key_to } => {
                MultiShardOps::move_value::<K, V, L>(&mut lock_guards.write, key_from, key_to);
            }
            Self::Remove { key } => {
                let shard = lock_guards.write_guard(key);
                ShardOps::remove_entry::<K, V>(shard, key);
            }
            Self::RemoveIf { key, condition } => {
                let shard = lock_guards.write_guard(key);
                ShardOps::remove_if(shard, key, |k, v| condition(k, v, state));
            }
            Self::SwapValue { key_a, key_b } => {
                MultiShardOps::swap_value::<K, V, L>(&mut lock_guards.write, key_a, key_b);
            }
            Self::Update { key, transform } => {
                let shard = lock_guards.write_guard(key);
                ShardOps::update(shard, key, |k, v_opt| transform(k, v_opt, state));
            }
        }
    }
}
