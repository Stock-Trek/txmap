use crate::{
    immediate::ops::{
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
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
};
use std::{hash::Hash, marker::PhantomData};

pub(crate) enum ImmediateOp<'tx, K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    Get(GetOp<'tx, K, V, STATE>),
    InsertDefault(InsertDefaultOp<K>),
    InsertDefaultIfAbsent(InsertDefaultIfAbsentOp<K>),
    InsertWith(InsertWithOp<'tx, K, V, STATE>),
    InsertWithIfAbsent(InsertWithIfAbsentOp<'tx, K, V, STATE>),
    Modify(ModifyOp<'tx, K, V, STATE>),
    MoveValue(MoveValueOp<K>),
    Remove(RemoveOp<'tx, K, V, STATE>),
    RemoveIf(RemoveIfOp<'tx, K, V, STATE>),
    SwapValue(SwapValueOp<K>),
    Update(UpdateOp<'tx, K, V, STATE>),
    #[doc(hidden)]
    _Phantom(PhantomData<L>),
}

impl<'tx, K, V, L, STATE> ImmediateOp<'tx, K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    pub fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        match self {
            Self::Get(op) => (op.key.shard_index.bitmask(), BitMask::ZERO),
            Self::InsertDefault(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::InsertDefaultIfAbsent(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::InsertWith(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::InsertWithIfAbsent(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::Modify(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::MoveValue(op) => (
                BitMask::ZERO,
                op.key_from.shard_index.bitmask() | op.key_to.shard_index.bitmask(),
            ),
            Self::Remove(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::RemoveIf(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::SwapValue(op) => (
                BitMask::ZERO,
                op.key_a.shard_index.bitmask() | op.key_b.shard_index.bitmask(),
            ),
            Self::Update(op) => (BitMask::ZERO, op.key.shard_index.bitmask()),
            Self::_Phantom(_) => unreachable!(),
        }
    }
}

impl<'tx, K, V, L, STATE> ImmediateOp<'tx, K, V, L, STATE>
where
    K: Clone + Hash + Eq,
    V: Default,
    L: LockPolicy,
{
    pub fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        match self {
            Self::Get(op) => GetOpApply::apply(&op.key, &*op.get, lock_guards, state),
            Self::InsertDefault(op) => InsertDefaultOpApply::apply(&op.key, lock_guards),
            Self::InsertDefaultIfAbsent(op) => {
                InsertDefaultIfAbsentOpApply::apply(&op.key, lock_guards)
            }
            Self::InsertWith(op) => {
                InsertWithOpApply::apply(&op.key, &*op.value_generator, lock_guards, state)
            }
            Self::InsertWithIfAbsent(op) => {
                InsertWithIfAbsentOpApply::apply(&op.key, &*op.value_generator, lock_guards, state)
            }
            Self::Modify(op) => ModifyOpApply::apply(&op.key, &*op.mutate, lock_guards, state),
            Self::MoveValue(op) => {
                MoveValueOpApply::apply::<K, V, L>(&op.key_from, &op.key_to, lock_guards)
            }
            Self::Remove(op) => RemoveOpApply::apply(&op.key, &*op.on_remove, lock_guards, state),
            Self::RemoveIf(op) => {
                RemoveIfOpApply::apply(&op.key, &*op.condition, lock_guards, state)
            }
            Self::SwapValue(op) => {
                SwapValueOpApply::apply::<K, V, L>(&op.key_a, &op.key_b, lock_guards)
            }
            Self::Update(op) => UpdateOpApply::apply(&op.key, &*op.transform, lock_guards, state),
            Self::_Phantom(_) => unreachable!(),
        }
    }
}
