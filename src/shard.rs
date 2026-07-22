use hashbrown::HashTable;

pub(crate) type Shard<K, V> = HashTable<(K, V)>;
