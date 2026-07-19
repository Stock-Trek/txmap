use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct SwapValueOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    a_index: u8,
    b_index: u8,
    a: K,
    b: K,
    _phantom: PhantomData<V>,
}

impl<K, V> SwapValueOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new(indexer: &Indexer, a: K, b: K) -> Self {
        let a_index = indexer.index(&a);
        let b_index = indexer.index(&b);
        Self {
            guards_bitmask: (1 << a_index) | (1 << b_index),
            a_index,
            b_index,
            a,
            b,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> OpTrait<K, V> for SwapValueOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let a_guard = mutex_guards
            .get_mut(self.a_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        let a_value = a_guard.remove(&self.a);
        drop(a_guard);

        let b_guard = mutex_guards
            .get_mut(self.b_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        let b_value = b_guard.remove(&self.b);
        drop(b_guard);

        if self.a_index == self.b_index {
            let guard = mutex_guards
                .get_mut(self.a_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            if let Some(v) = b_value {
                guard.insert(self.a.clone(), v);
            }
            if let Some(v) = a_value {
                guard.insert(self.b.clone(), v);
            }
        } else {
            // Insert a's old value at b's position and vice versa
            let a_guard = mutex_guards
                .get_mut(self.a_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            if let Some(v) = b_value {
                a_guard.insert(self.a.clone(), v);
            }
            drop(a_guard);

            let b_guard = mutex_guards
                .get_mut(self.b_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            if let Some(v) = a_value {
                b_guard.insert(self.b.clone(), v);
            }
        }
    }
}
