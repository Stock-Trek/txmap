use crate::{
    locks::lock_policy::LockPolicy,
    new_types::{BitMask, ShardIndex},
};
use hashbrown::HashTable;
use intmap::IntMap;

pub(crate) struct Custodian<L, K, V>
where
    L: LockPolicy,
{
    pub(crate) shard_count: u8,
    pub(crate) shards: Vec<L::Lock<Shard<K, V>>>,
}

type Shard<K, V> = HashTable<(K, V)>;

impl<L, K, V> Custodian<L, K, V>
where
    L: LockPolicy,
{
    pub fn new(shard_count: u8) -> Self {
        let mut shards = Vec::with_capacity(shard_count as usize);
        for _ in 0..shard_count {
            shards.push(L::new(HashTable::new()));
        }
        Self {
            shard_count,
            shards,
        }
    }
    pub fn all_guards(&self) -> IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>> {
        let all_guards_bitmask = if self.shard_count == 128 {
            !0u128
        } else {
            (1 << self.shard_count) - 1
        };
        self.guards(BitMask(all_guards_bitmask))
    }
    pub fn guards(&self, bitmask: BitMask) -> IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>> {
        let mut guards = IntMap::new();
        for i in 0..self.shard_count {
            let is_lock_required = ((bitmask.0 >> i) & 1) == 1;
            if is_lock_required {
                let shard_lock = &self.shards[i as usize];
                let guard = L::write(shard_lock);
                guards.insert(i, guard);
            };
        }
        guards
    }
    pub fn guard_at(&self, shard_index: ShardIndex) -> L::WriteGuard<'_, HashTable<(K, V)>> {
        let shard_lock = &self.shards[shard_index.0 as usize];
        L::write(shard_lock)
    }
}
