# API comparison: `TxMap` vs `std::collections::HashMap`

The goal is for `TxMap` to be as close to a drop-in replacement for [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) as possible given this library's transactional guarantees.
This document compares the two API surfaces: what is the same, what is different and what is available in each one but not the other.

## Fundamental design difference

|                           | `std::collections::HashMap`    | `TxMap`                                         |
|---------------------------|--------------------------------|-------------------------------------------------|
| Mutating operations take  | `&mut self` (exclusive borrow) | `&self` (internal mutability)                   |
| Concurrency               | Not `Sync`                     | `Sync`; fine-grained per-shard locking          |
| References escape the map | Yes (`&V`, `&mut V`, `Entry`)  | No (locks are released when the method returns) |

Because `TxMap` is internally mutable and lock-based, it can never return references into its storage: a lock guard is dropped when the method returns, so a `&V` or `&mut V` would dangle. Every std method that returns a reference has a closure-based counterpart in `TxMap` (see [*Different*](#different-api-same-name-different-signature-or-semantics) below).

## Same API (same name, same or equivalent semantics)

| API                                  | Notes                                                               |
|--------------------------------------|---------------------------------------------------------------------|
| `new()`                              |                                                                     |
| `Default`                            |                                                                     |
| `insert(k, v) -> Option<V>`          | Returns the previous value                                          |
| `remove(&k) -> Option<V>`            |                                                                     |
| `remove_entry(&k) -> Option<(K, V)>` |                                                                     |
| `contains_key(&k) -> bool`           |                                                                     |
| `len() -> usize`                     |                                                                     |
| `is_empty() -> bool`                 |                                                                     |
| `clear()`                            |                                                                     |
| `iter() -> (&K, &V)`                 |                                                                     |
| `keys() -> &K`                       |                                                                     |
| `values() -> &V`                     |                                                                     |
| `drain() -> (K, V)`                  | Dropping the iterator mid-way removes all remaining entries in both |
| `capacity() -> usize`                | `TxMap`'s is approximate (see *Different*)                          |
| `reserve(additional)`                | Distributed evenly across shards                                    |
| `try_reserve(additional)`            |                                                                     |
| `shrink_to_fit()`                    |                                                                     |
| `shrink_to(min_capacity)`            |                                                                     |
| `hasher() -> &S`                     |                                                                     |
| `IntoIterator` (owned, `&`, `&mut`)  | Owned iteration is eager in `TxMap`                                 |
| `Extend<(K, V)>`, `Extend<(&K, &V)>` |                                                                     |
| `FromIterator<(K, V)>`               |                                                                     |
| `From<[(K, V); N]>`                  |                                                                     |
| `Clone`                              | Both require `V: Clone`                                             |
| `PartialEq`, `Eq`                    | Element-wise comparison, order-independent                          |
| `Debug`                              |                                                                     |
| serde `Serialize`/`Deserialize`      | Via the `serde` feature                                             |

## Different API (same name, different signature or semantics)

| std `HashMap`                                                       | `TxMap`                                                              | Why                                                                                                                       |
|---------------------------------------------------------------------|----------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| `get(&k) -> Option<&V>`                                             | `get_with(&k, fn(&V) -> R) -> Option<R>`, `get_copied`, `get_cloned` | A `&V` cannot escape the lock; read the value inside a closure instead                                                    |
| `get_key_value(&k) -> Option<(&K, &V)>`                             | `get_with` (key is passed to the closure)                            | Same reference limitation                                                                                                 |
| `get_mut(&k) -> Option<&mut V>`                                     | `modify(&k, fn(&K, &mut V)) -> bool`                                 | `&mut V` cannot escape the lock; mutate inside a closure                                                                  |
| `entry(k) -> Entry`                                                 | transactions / `update(&k, fn(&K, Option<&V>) -> Option<V>)`         | `Entry` borrows the map; `TxMap` provides atomic composable transactions instead                                          |
| `try_insert(k, v) -> Result<&mut V, OccupiedError>`                 | `insert_with_if_absent(k, fn() -> V) -> bool`                        | `&mut V` cannot escape the lock                                                                                           |
| `iter_mut()`                                                        | `modify` / `update` / transactions                                   | `&mut V` cannot escape the lock                                                                                           |
| `values_mut()`                                                      | `modify` / `update` / transactions                                   | `&mut V` cannot escape the lock                                                                                           |
| `retain(FnMut(&K, &mut V) -> bool)`                                 | `retain(Fn(&K, &V) -> bool)`                                         | The predicate cannot mutate in `TxMap`; use transactions for that                                                         |
| `capacity()`                                                        | `capacity()`                                                         | Sum of per-shard capacities; each shard rounds up to its allocation granularity so the total is an upper bound, not exact |
| `try_reserve(...) -> Result<(), std::collections::TryReserveError>` | `try_reserve(...) -> Result<(), txmap::result::TryReserveError>`     | std's error type is opaque on stable Rust, so `TxMap` defines a matching public error type                                |
| `Index<&Q> for HashMap` (returns `&V`, panics if absent)            | not implemented                                                      | `&V` cannot escape the lock; use `get_with(...).expect(...)` or `contains_key`                                            |

## Available in std but not in `TxMap`

| std `HashMap`                                | Status in `TxMap`                                     | Workaround                                                                                                                      |
|----------------------------------------------|-------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------|
| `get`, `get_key_value`                       | Not implementable (returns `&V`/`(&K, &V)`)           | `get_with`, `get_copied`, `get_cloned`                                                                                          |
| `get_mut`, `iter_mut`, `values_mut`          | Not implementable (returns `&mut V`)                  | `modify`, `update`, transactions                                                                                                |
| `entry`, `try_entry`                         | Not implementable (returns `Entry` borrowing the map) | `update`, transactions, guards                                                                                                  |
| `try_insert`                                 | Not implementable (returns `&mut V`)                  | `insert_with_if_absent`                                                                                                         |
| `Index<&Q>`                                  | Not implementable (returns `&V`)                      | `get_with(...).expect(...)`                                                                                                     |
| `Hash`                                       | Intentionally omitted                                 | std removed `Hash` from `HashMap` because the hash of an order-dependent iteration cannot satisfy `a == b ⇒ hash(a) == hash(b)` |
| `get_many_mut`, `get_disjoint_mut` (nightly) | Not applicable                                        | `modify`/`update` in a transaction                                                                                              |
| `insert_unique_unchecked` (nightly)          | Not applicable                                        | `insert`                                                                                                                        |
| `remove_if` (nightly)                        | Already stable in `TxMap`                             | —                                                                                                                               |
| `extract_if` (nightly)                       | Not available                                         | `retain`, or `drain` + filter                                                                                                   |

## Available in `TxMap` but not in std `HashMap`

| `TxMap`                                                 | Description                                                            |
|---------------------------------------------------------|------------------------------------------------------------------------|
| `get_with`, `get_copied`, `get_cloned`                  | Lock-safe reads (return values/closures instead of references)         |
| `insert_with_if_absent`                                 | Insert only if absent, without re-hashing the key twice                |
| `modify`                                                | In-place mutation of an existing value                                 |
| `update`                                                | Insert/replace/remove in one operation based on the current value      |
| `move_value`                                            | Atomically move a value from one key to another (cross-shard)          |
| `swap_value`                                            | Atomically swap the values of two keys (cross-shard)                   |
| `fold`                                                  | Accumulate over all entries with a conversion step                     |
| `remove_if`                                             | Stable equivalent of std's nightly `remove_if`                         |
| `immediate_tx`, `prepared_tx`                           | Composable, atomic, multi-key transactions with guards                 |
| `TxMapBuilder`, `Shards`, `MutexPolicy`, `RwLockPolicy` | Configuration: shard count, lock policy, capacity, hasher              |
| All operations take `&self`                             | Shared references suffice; no `&mut` needed, maps can be shared freely |
