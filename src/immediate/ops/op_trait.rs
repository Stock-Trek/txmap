use crate::{
    lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy, new_types::BitMask, ops::*,
};
use std::hash::Hash;

pub(crate) enum Op<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    Get(super::get_op::GetOp<'tx, K, V, STATE>),
    InsertDefault(super::insert_default_op::InsertDefaultOp<K>),
    InsertDefaultIfAbsent(super::insert_default_if_absent_op::InsertDefaultIfAbsentOp<K>),
    InsertWith(super::insert_with_op::InsertWithOp<'tx, K, V, STATE>),
    InsertWithIfAbsent(super::insert_with_if_absent_op::InsertWithIfAbsentOp<'tx, K, V, STATE>),
    Remove(super::remove_op::RemoveOp<'tx, K, V, STATE>),
    RemoveIf(super::remove_if_op::RemoveIfOp<'tx, K, V, STATE>),
    Update(super::update_op::UpdateOp<'tx, K, V, STATE>),
    Modify(super::modify_op::ModifyOp<'tx, K, V, STATE>),
    MoveValue(super::move_value_op::MoveValueOp<K>),
    SwapValue(super::swap_value_op::SwapValueOp<K>),
}

impl<'tx, K, V, STATE> Op<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub(crate) fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        match self {
            Op::Get(op) => GetOpFn::read_write_bitmasks(&op.key),
            Op::InsertDefault(op) => InsertDefaultFn::read_write_bitmasks(&op.key),
            Op::InsertDefaultIfAbsent(op) => InsertDefaultIfAbsentFn::read_write_bitmasks(&op.key),
            Op::InsertWith(op) => InsertWithFn::read_write_bitmasks(&op.key),
            Op::InsertWithIfAbsent(op) => InsertWithIfAbsentFn::read_write_bitmasks(&op.key),
            Op::Remove(op) => RemoveFn::read_write_bitmasks(&op.key),
            Op::RemoveIf(op) => RemoveIfFn::read_write_bitmasks(&op.key),
            Op::Update(op) => UpdateFn::read_write_bitmasks(&op.key),
            Op::Modify(op) => ModifyFn::read_write_bitmasks(&op.key),
            Op::MoveValue(op) => MoveValueFn::read_write_bitmasks(&op.key_from, &op.key_to),
            Op::SwapValue(op) => SwapValueFn::read_write_bitmasks(&op.key_a, &op.key_b),
        }
    }

    pub(crate) fn apply<L: LockPolicy>(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone,
        V: Default,
    {
        match self {
            Op::Get(op) => GetOpFn::apply(&op.key, &op.get, lock_guards, state),
            Op::InsertDefault(op) => InsertDefaultFn::apply(&op.key, lock_guards),
            Op::InsertDefaultIfAbsent(op) => InsertDefaultIfAbsentFn::apply(&op.key, lock_guards),
            Op::InsertWith(op) => {
                InsertWithFn::apply(&op.key, &op.value_generator, lock_guards, state)
            }
            Op::InsertWithIfAbsent(op) => {
                InsertWithIfAbsentFn::apply(&op.key, &op.value_generator, lock_guards, state)
            }
            Op::Remove(op) => RemoveFn::apply(&op.key, &op.on_remove, lock_guards, state),
            Op::RemoveIf(op) => RemoveIfFn::apply(&op.key, &op.condition, lock_guards, state),
            Op::Update(op) => UpdateFn::apply(&op.key, &op.transform, lock_guards, state),
            Op::Modify(op) => ModifyFn::apply(&op.key, &op.mutate, lock_guards, state),
            Op::MoveValue(op) => MoveValueFn::apply(&op.key_from, &op.key_to, lock_guards),
            Op::SwapValue(op) => SwapValueFn::apply(&op.key_a, &op.key_b, lock_guards),
        }
    }
}
