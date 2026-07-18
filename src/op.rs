use crate::ops::{
    clear_op::ClearOp, insert_default_op::InsertDefaultOp, insert_with_op::InsertWithOp,
    map_op::MapOp, map_peek_op::MapPeekOp, modify_op::ModifyOp,
    modify_or_default_op::ModifyOrDefaultOp, modify_or_insert_with_op::ModifyOrInsertWithOp,
    modify_peek_op::ModifyPeekOp, modify_peek_or_default_op::ModifyPeekOrDefaultOp,
    modify_peek_or_insert_with_op::ModifyPeekOrInsertWithOp, move_value_op::MoveValueOp,
    mut_op::MutOp, remove_any_if_op::RemoveAnyIfOp, remove_if_op::RemoveIfOp, remove_op::RemoveOp,
    retain_any_if_op::RetainAnyIfOp, retain_if_op::RetainIfOp, retain_op::RetainOp,
    swap_value_op::SwapValueOp,
};
use std::hash::Hash;

pub(crate) enum Op<K, V>
where
    K: Clone + Hash + Eq,
{
    InsertWith(InsertWithOp<K, V>),
    InsertDefault(InsertDefaultOp<K, V>),
    Modify(ModifyOp<K, V>),
    ModifyPeek(ModifyPeekOp<K, V>),
    ModifyOrInsertWith(ModifyOrInsertWithOp<K, V>),
    ModifyPeekOrInsertWith(ModifyPeekOrInsertWithOp<K, V>),
    ModifyOrDefault(ModifyOrDefaultOp<K, V>),
    ModifyPeekOrDefault(ModifyPeekOrDefaultOp<K, V>),
    Map(MapOp<K, V>),
    MapPeek(MapPeekOp<K, V>),
    Mut(MutOp<K, V>),
    SwapValue(SwapValueOp<K, V>),
    MoveValue(MoveValueOp<K, V>),
    Remove(RemoveOp<K, V>),
    RemoveIf(RemoveIfOp<K, V>),
    Retain(RetainOp<K, V>),
    RetainIf(RetainIfOp<K, V>),
    Clear(ClearOp<K, V>),
    RemoveAnyIf(RemoveAnyIfOp<K, V>),
    RetainAnyIf(RetainAnyIfOp<K, V>),
}

impl<K, V> Op<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn guards_bitmask(&self) -> u128 {
        match self {
            Self::Map(op) => op.guards_bitmask,
            Self::MapPeek(op) => op.guards_bitmask,
            Self::Mut(op) => op.guards_bitmask,
            Self::InsertWith(op) => op.guards_bitmask,
            Self::InsertDefault(op) => op.guards_bitmask,
            Self::Modify(op) => op.guards_bitmask,
            Self::ModifyPeek(op) => op.guards_bitmask,
            Self::ModifyOrInsertWith(op) => op.guards_bitmask,
            Self::ModifyPeekOrInsertWith(op) => op.guards_bitmask,
            Self::ModifyOrDefault(op) => op.guards_bitmask,
            Self::ModifyPeekOrDefault(op) => op.guards_bitmask,
            Self::SwapValue(op) => op.guards_bitmask,
            Self::MoveValue(op) => op.guards_bitmask,
            Self::Remove(op) => op.guards_bitmask,
            Self::RemoveIf(op) => op.guards_bitmask,
            Self::Retain(op) => op.guards_bitmask,
            Self::RetainIf(op) => op.guards_bitmask,
            Self::Clear(op) => op.guards_bitmask,
            Self::RemoveAnyIf(op) => op.guards_bitmask,
            Self::RetainAnyIf(op) => op.guards_bitmask,
        }
    }
}
