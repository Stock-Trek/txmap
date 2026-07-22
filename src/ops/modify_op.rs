use crate::{
    indexed_key::IndexedKey, locks::lock_policy::LockPolicy, new_types::BitMask,
    ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct ModifyOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    #[allow(clippy::type_complexity)]
    mutate: Box<dyn Fn(&K, &mut V, &P)>,
}

impl<K, V, P> ModifyOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<M>(shard_count: u8, key: K, mutate: M) -> Self
    where
        M: Fn(&K, &mut V, &P) + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            mutate: Box::new(mutate),
        }
    }
}

impl<K, V> ModifyOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<M>(shard_count: u8, key: K, mutate: M) -> Self
    where
        M: Fn(&K, &mut V) + 'static,
    {
        Self::new_with_params(shard_count, key, move |k, v, _| mutate(k, v))
    }
}

impl<L, K, V, P> OpTrait<L, K, V, P> for ModifyOp<K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply<'guards>(
        &self,
        mutex_guards: &'guards mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
        params: &P,
    ) {
        let mutex_guard = mutex_guards
            .get_mut(self.indexed_key.1.0)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some(mut_entry) =
            mutex_guard.find_mut(self.indexed_key.0.0, |x| x.0 == self.indexed_key.3)
        {
            (self.mutate)(&mut_entry.0, &mut mut_entry.1, params)
        }
    }
}
