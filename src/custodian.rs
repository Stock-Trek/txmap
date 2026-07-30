use crate::{
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::{BitMask, ShardCount, ShardIndex},
    shard::Shard,
};
use hashbrown::HashTable;
use intmap::IntMap;

pub(crate) struct Custodian<K, V, L>
where
    L: LockPolicy,
{
    pub(crate) shard_count: ShardCount,
    pub(crate) shards: Vec<L::Lock<Shard<K, V>>>,
}

impl<K, V, L> Custodian<K, V, L>
where
    L: LockPolicy,
{
    pub fn new(shard_count: ShardCount) -> Self {
        let mut shards = Vec::with_capacity(shard_count.0 as usize);
        for _ in 0..shard_count.0 {
            shards.push(L::new(HashTable::new()));
        }
        Self {
            shard_count,
            shards,
        }
    }
    pub fn all_read_guards(&self) -> IntMap<u8, L::ReadGuard<'_, Shard<K, V>>> {
        self.lock_guards(self.all_bitmask(), BitMask::ZERO).read
    }
    pub fn all_write_guards(&self) -> IntMap<u8, L::WriteGuard<'_, Shard<K, V>>> {
        self.lock_guards(BitMask::ZERO, self.all_bitmask()).write
    }
    fn all_bitmask(&self) -> BitMask {
        let bitmask = if self.shard_count.0 == 128 {
            !0u128
        } else {
            (1 << self.shard_count.0) - 1
        };
        BitMask(bitmask)
    }
    pub fn lock_guards(&self, read: BitMask, write: BitMask) -> LockGuards<'_, K, V, L> {
        let mut read_guards = IntMap::new();
        let mut write_guards = IntMap::new();
        for i in 0..self.shard_count.0 {
            let bitmask = ShardIndex(i).bitmask();
            let shard_lock = &self.shards[i as usize];
            if (write & bitmask) != BitMask::ZERO {
                let write_guard = L::write(shard_lock);
                write_guards.insert(i, write_guard);
            } else if (read & bitmask) != BitMask::ZERO {
                let read_guard = L::read(shard_lock);
                read_guards.insert(i, read_guard);
            };
        }
        LockGuards {
            read: read_guards,
            write: write_guards,
            write_bitmask: write,
        }
    }
    pub fn write_guards(&self, write: BitMask) -> IntMap<u8, L::WriteGuard<'_, Shard<K, V>>> {
        let mut write_guards = IntMap::new();
        for i in 0..self.shard_count.0 {
            let bitmask = ShardIndex(i).bitmask();
            if (write & bitmask) != BitMask::ZERO {
                let shard_lock = &self.shards[i as usize];
                let write_guard = L::write(shard_lock);
                write_guards.insert(i, write_guard);
            };
        }
        write_guards
    }
    pub fn read_guard_at(&self, shard_index: ShardIndex) -> L::ReadGuard<'_, Shard<K, V>> {
        let shard_lock = &self.shards[shard_index.0 as usize];
        L::read(shard_lock)
    }
    #[allow(dead_code)]
    pub fn write_guard_at(&self, shard_index: ShardIndex) -> L::WriteGuard<'_, Shard<K, V>> {
        let shard_lock = &self.shards[shard_index.0 as usize];
        L::write(shard_lock)
    }
}
