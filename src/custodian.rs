use crate::{
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::{BitMask, MAX_SHARDS, ShardCount, ShardIndex},
    shard::Shard,
};
use crossbeam_utils::CachePadded;
use hashbrown::HashTable;

pub(crate) struct Custodian<K, V, L>
where
    L: LockPolicy,
{
    pub(crate) shard_count: ShardCount,
    pub(crate) shards: Vec<CachePadded<L::Lock<Shard<K, V>>>>,
}

impl<K, V, L> Custodian<K, V, L>
where
    L: LockPolicy,
{
    pub fn new(shard_count: ShardCount, capacity: usize) -> Self {
        let mut shards = Vec::with_capacity(shard_count.0 as usize);
        let capacity_per_shard = capacity.div_ceil(shard_count.0 as usize);
        for _ in 0..shard_count.0 {
            shards.push(CachePadded::new(L::new(HashTable::with_capacity(
                capacity_per_shard,
            ))));
        }
        Self {
            shard_count,
            shards,
        }
    }
    pub fn all_read_guards(&self) -> Vec<L::ReadGuard<'_, Shard<K, V>>> {
        let mut guards = Vec::with_capacity(self.shard_count.0 as usize);
        for shard in &self.shards {
            guards.push(L::read(shard));
        }
        guards
    }
    pub fn all_write_guards(&self) -> Vec<L::WriteGuard<'_, Shard<K, V>>> {
        let mut guards = Vec::with_capacity(self.shard_count.0 as usize);
        for shard in &self.shards {
            guards.push(L::write(shard));
        }
        guards
    }
    pub fn lock_guards(&self, read: BitMask, write: BitMask) -> LockGuards<'_, K, V, L> {
        let mut read_guards = std::array::from_fn(|_| None);
        let mut write_guards = std::array::from_fn(|_| None);

        let mut bits = (read | write).0;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            let shard_lock = &self.shards[i];
            let bit = BitMask(1u128 << i);
            if (write & bit) != BitMask::ZERO {
                write_guards[i] = Some(L::write(shard_lock));
            } else {
                read_guards[i] = Some(L::read(shard_lock));
            }
        }
        LockGuards {
            read: read_guards,
            write: write_guards,
            write_bitmask: write,
        }
    }
    pub fn write_guards(
        &self,
        write: BitMask,
    ) -> [Option<L::WriteGuard<'_, Shard<K, V>>>; MAX_SHARDS] {
        let mut write_guards = std::array::from_fn(|_| None);

        let mut bits = write.0;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            let shard_lock = &self.shards[i];
            write_guards[i] = Some(L::write(shard_lock));
        }
        write_guards
    }
    pub fn read_guard_at(&self, shard_index: ShardIndex) -> L::ReadGuard<'_, Shard<K, V>> {
        let shard_lock = &self.shards[shard_index.0 as usize];
        L::read(shard_lock)
    }
    pub fn write_guard_at(&self, shard_index: ShardIndex) -> L::WriteGuard<'_, Shard<K, V>> {
        let shard_lock = &self.shards[shard_index.0 as usize];
        L::write(shard_lock)
    }
}
