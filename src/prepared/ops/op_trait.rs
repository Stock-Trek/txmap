use crate::{
    lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy, new_types::BitMask, ops::*,
};
use std::hash::Hash;

pub(crate) enum Op<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    Get(super::get_op::GetOp<'tx, K, V, KEYS, PARAMS, STATE>),
    InsertDefault(super::insert_default_op::InsertDefaultOp<'tx, K, KEYS>),
    InsertDefaultIfAbsent(
        super::insert_default_if_absent_op::InsertDefaultIfAbsentOp<'tx, K, KEYS>,
    ),
    InsertWith(super::insert_with_op::InsertWithOp<'tx, K, V, KEYS, PARAMS, STATE>),
    InsertWithIfAbsent(
        super::insert_with_if_absent_op::InsertWithIfAbsentOp<'tx, K, V, KEYS, PARAMS, STATE>,
    ),
    Remove(super::remove_op::RemoveOp<'tx, K, V, KEYS, PARAMS, STATE>),
    RemoveIf(super::remove_if_op::RemoveIfOp<'tx, K, V, KEYS, PARAMS, STATE>),
    Update(super::update_op::UpdateOp<'tx, K, V, KEYS, PARAMS, STATE>),
    Modify(super::modify_op::ModifyOp<'tx, K, V, KEYS, PARAMS, STATE>),
    MoveValue(super::move_value_op::MoveValueOp<'tx, K, KEYS>),
    SwapValue(super::swap_value_op::SwapValueOp<'tx, K, KEYS>),
}

impl<'tx, K, V, KEYS, PARAMS, STATE> Op<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    pub(crate) fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask) {
        match self {
            Op::Get(op) => GetOpFn::read_write_bitmasks(op.key_selector.get(keys)),
            Op::InsertDefault(op) => {
                InsertDefaultFn::read_write_bitmasks(op.key_selector.get(keys))
            }
            Op::InsertDefaultIfAbsent(op) => {
                InsertDefaultIfAbsentFn::read_write_bitmasks(op.key_selector.get(keys))
            }
            Op::InsertWith(op) => InsertWithFn::read_write_bitmasks(op.key_selector.get(keys)),
            Op::InsertWithIfAbsent(op) => {
                InsertWithIfAbsentFn::read_write_bitmasks(op.key_selector.get(keys))
            }
            Op::Remove(op) => RemoveFn::read_write_bitmasks(op.key_selector.get(keys)),
            Op::RemoveIf(op) => RemoveIfFn::read_write_bitmasks(op.key_selector.get(keys)),
            Op::Update(op) => UpdateFn::read_write_bitmasks(op.key_selector.get(keys)),
            Op::Modify(op) => ModifyFn::read_write_bitmasks(op.key_selector.get(keys)),
            Op::MoveValue(op) => MoveValueFn::read_write_bitmasks(
                op.key_selector_from.get(keys),
                op.key_selector_to.get(keys),
            ),
            Op::SwapValue(op) => SwapValueFn::read_write_bitmasks(
                op.key_selector_a.get(keys),
                op.key_selector_b.get(keys),
            ),
        }
    }

    pub(crate) fn apply<L: LockPolicy>(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Clone,
        V: Default,
    {
        match self {
            Op::Get(op) => {
                let key = op.key_selector.get(keys);
                GetOpFn::apply_prepared(key, &op.get, lock_guards, keys, params, state)
            }
            Op::InsertDefault(op) => InsertDefaultFn::apply(op.key_selector.get(keys), lock_guards),
            Op::InsertDefaultIfAbsent(op) => {
                InsertDefaultIfAbsentFn::apply(op.key_selector.get(keys), lock_guards)
            }
            Op::InsertWith(op) => {
                let key = op.key_selector.get(keys);
                InsertWithFn::apply_prepared(
                    key,
                    &op.value_generator,
                    lock_guards,
                    keys,
                    params,
                    state,
                )
            }
            Op::InsertWithIfAbsent(op) => {
                let key = op.key_selector.get(keys);
                InsertWithIfAbsentFn::apply_prepared(
                    key,
                    &op.value_generator,
                    lock_guards,
                    keys,
                    params,
                    state,
                )
            }
            Op::Remove(op) => {
                let key = op.key_selector.get(keys);
                RemoveFn::apply_prepared(key, &op.on_remove, lock_guards, keys, params, state)
            }
            Op::RemoveIf(op) => {
                let key = op.key_selector.get(keys);
                RemoveIfFn::apply_prepared(key, &op.condition, lock_guards, keys, params, state)
            }
            Op::Update(op) => {
                let key = op.key_selector.get(keys);
                UpdateFn::apply_prepared(key, &op.transform, lock_guards, keys, params, state)
            }
            Op::Modify(op) => {
                let key = op.key_selector.get(keys);
                ModifyFn::apply_prepared(key, &op.mutate, lock_guards, keys, params, state)
            }
            Op::MoveValue(op) => MoveValueFn::apply(
                op.key_selector_from.get(keys),
                op.key_selector_to.get(keys),
                lock_guards,
            ),
            Op::SwapValue(op) => SwapValueFn::apply(
                op.key_selector_a.get(keys),
                op.key_selector_b.get(keys),
                lock_guards,
            ),
        }
    }
}
