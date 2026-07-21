use crate::{finishers::finisher_trait::FinisherTrait, new_types::BitMask};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::marker::PhantomData;

pub(crate) struct Finisher<K, V, F>
where
    F: FinisherTrait<K, V>,
{
    finisher: F,
    _phantom_k: PhantomData<K>,
    _phantom_v: PhantomData<V>,
}

impl<K, V, F> Finisher<K, V, F>
where
    F: FinisherTrait<K, V>,
{
    pub fn new(finisher: F) -> Self {
        Self {
            finisher,
            _phantom_k: PhantomData,
            _phantom_v: PhantomData,
        }
    }
    pub fn guards_bitmask(&self) -> BitMask {
        self.finisher.guards_bitmask()
    }
    pub fn finish(&self, mutex_guards: &IntMap<u8, MutexGuard<HashTable<(K, V)>>>) -> F::Output {
        self.finisher.to_result(mutex_guards)
    }
}
