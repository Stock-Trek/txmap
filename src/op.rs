use crate::ops::{
    insert_default_op::InsertDefaultOp, insert_with_op::InsertWithOp, map_op::MapOp,
    map_peek_op::MapPeekOp, modify_op::ModifyOp, modify_peek_op::ModifyPeekOp, mut_op::MutOp,
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
    Map(MapOp<K, V>),
    MapPeek(MapPeekOp<K, V>),
    Mut(MutOp<K, V>),
}

impl<K, V> Op<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn guards_bitmask(&self) -> u128 {
        match self {
            Self::Map(map_op) => map_op.guards_bitmask,
            Self::Mut(mut_op) => mut_op.guards_bitmask,
        }
    }
}
