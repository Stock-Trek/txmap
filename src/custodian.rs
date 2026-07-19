use crate::shard_count::ShardCount;
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::{Mutex, MutexGuard};

pub(crate) struct Custodian<K, V> {
    shard_count: u8,
    shards: Vec<Shard<K, V>>,
}

type Shard<K, V> = Mutex<HashMap<K, V>>;

impl<K, V> Custodian<K, V> {
    pub fn new(shard_count: ShardCount) -> Self {
        let shard_count = u8::from(shard_count);
        let mut shards = Vec::with_capacity(shard_count as usize);
        for _ in 0..shard_count {
            shards.push(Mutex::new(HashMap::new()));
        }
        Self {
            shard_count,
            shards,
        }
    }
    pub fn all_guards(&self) -> IntMap<u8, MutexGuard<'_, HashMap<K, V>>> {
        let all_guards_bitmask = if self.shard_count == 128 {
            !0u128
        } else {
            (1 << self.shard_count) - 1
        };
        self.guards(all_guards_bitmask)
    }
    pub fn guards(&self, bitmask: u128) -> IntMap<u8, MutexGuard<'_, HashMap<K, V>>> {
        let mut guards = IntMap::new();
        for i in 0..self.shard_count {
            let is_lock_required = ((bitmask >> i) & 1) == 1;
            if is_lock_required {
                let guard = self.shards[i as usize].lock();
                guards.insert(i, guard);
            };
        }
        guards
    }
}
