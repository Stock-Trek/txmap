use crate::{
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::ops::{
        get_op::{GetOp, GetOpApply},
        insert_default_if_absent_op::{InsertDefaultIfAbsentOp, InsertDefaultIfAbsentOpApply},
        insert_default_op::{InsertDefaultOp, InsertDefaultOpApply},
        insert_with_if_absent_op::{InsertWithIfAbsentOp, InsertWithIfAbsentOpApply},
        insert_with_op::{InsertWithOp, InsertWithOpApply},
        modify_op::{ModifyOp, ModifyOpApply},
        move_value_op::{MoveValueOp, MoveValueOpApply},
        remove_if_op::{RemoveIfOp, RemoveIfOpApply},
        remove_op::{RemoveOp, RemoveOpApply},
        swap_value_op::{SwapValueOp, SwapValueOpApply},
        update_op::{UpdateOp, UpdateOpApply},
    },
};
use std::{hash::Hash, marker::PhantomData};

pub(crate) enum PreparedOp<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    Get(GetOp<'tx, K, V, KEYS, PARAMS, STATE>),
    InsertDefault(InsertDefaultOp<'tx, K, KEYS>),
    InsertDefaultIfAbsent(InsertDefaultIfAbsentOp<'tx, K, KEYS>),
    InsertWith(InsertWithOp<'tx, K, V, KEYS, PARAMS, STATE>),
    InsertWithIfAbsent(InsertWithIfAbsentOp<'tx, K, V, KEYS, PARAMS, STATE>),
    Modify(ModifyOp<'tx, K, V, KEYS, PARAMS, STATE>),
    MoveValue(MoveValueOp<'tx, K, KEYS>),
    Remove(RemoveOp<'tx, K, V, KEYS, PARAMS, STATE>),
    RemoveIf(RemoveIfOp<'tx, K, V, KEYS, PARAMS, STATE>),
    SwapValue(SwapValueOp<'tx, K, KEYS>),
    Update(UpdateOp<'tx, K, V, KEYS, PARAMS, STATE>),
    #[doc(hidden)]
    _Phantom(PhantomData<L>),
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> PreparedOp<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    pub fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask) {
        match self {
            Self::Get(op) => (
                op.key_selector.get(keys).shard_index.bitmask(),
                BitMask::ZERO,
            ),
            Self::InsertDefault(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::InsertDefaultIfAbsent(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::InsertWith(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::InsertWithIfAbsent(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::Modify(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::MoveValue(op) => (
                BitMask::ZERO,
                op.key_selector_from.get(keys).shard_index.bitmask()
                    | op.key_selector_to.get(keys).shard_index.bitmask(),
            ),
            Self::Remove(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::RemoveIf(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::SwapValue(op) => (
                BitMask::ZERO,
                op.key_selector_a.get(keys).shard_index.bitmask()
                    | op.key_selector_b.get(keys).shard_index.bitmask(),
            ),
            Self::Update(op) => (
                BitMask::ZERO,
                op.key_selector.get(keys).shard_index.bitmask(),
            ),
            Self::_Phantom(_) => unreachable!(),
        }
    }
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> PreparedOp<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq,
    V: Default,
    L: LockPolicy,
    STATE: Default,
{
    pub fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) {
        match self {
            Self::Get(op) => GetOpApply::apply(
                &*op.key_selector,
                &*op.get,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::InsertDefault(op) => {
                InsertDefaultOpApply::apply(&*op.key_selector, lock_guards, keys)
            }
            Self::InsertDefaultIfAbsent(op) => {
                InsertDefaultIfAbsentOpApply::apply(&*op.key_selector, lock_guards, keys)
            }
            Self::InsertWith(op) => InsertWithOpApply::apply(
                &*op.key_selector,
                &*op.value_generator,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::InsertWithIfAbsent(op) => InsertWithIfAbsentOpApply::apply(
                &*op.key_selector,
                &*op.value_generator,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::Modify(op) => ModifyOpApply::apply(
                &*op.key_selector,
                &*op.mutate,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::MoveValue(op) => MoveValueOpApply::apply::<K, V, L, KEYS>(
                &*op.key_selector_from,
                &*op.key_selector_to,
                lock_guards,
                keys,
            ),
            Self::Remove(op) => RemoveOpApply::apply(
                &*op.key_selector,
                &*op.on_remove,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::RemoveIf(op) => RemoveIfOpApply::apply(
                &*op.key_selector,
                &*op.condition,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::SwapValue(op) => SwapValueOpApply::apply::<K, V, L, KEYS>(
                &*op.key_selector_a,
                &*op.key_selector_b,
                lock_guards,
                keys,
            ),
            Self::Update(op) => UpdateOpApply::apply(
                &*op.key_selector,
                &*op.transform,
                lock_guards,
                keys,
                params,
                state,
            ),
            Self::_Phantom(_) => unreachable!(),
        }
    }
}
