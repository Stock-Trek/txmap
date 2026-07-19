# TxMap

[![crates.io](https://img.shields.io/crates/v/txmap)](https://crates.io/crates/txmap)
[![docs.rs](https://img.shields.io/docsrs/txmap)](https://docs.rs/txmap)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A concurrent transactional hash map for Rust with fine-grained locking and internal mutability.

`TxMap` partitions stored key-value pairs across multiple shards, each protected by its own `parking_lot::Mutex`. Read and write operations acquire locks only on the shards they need, maximizing concurrency. Transactional operations group multiple reads and writes into atomic units, and support parameterized closures for flexible, reusable logic.

## Features

- **Concurrent access** — fine-grained shard-level locking; operations lock only the shards they touch.
- **Transactions** — atomic, composable batches of reads and writes.
- **Parameterized transactions** — pass user-defined parameters into transaction closures.
- **Guards/conditions** — declarative preconditions that must hold for a transaction to succeed.
- **Flexible operations** — modify, map (insert/update/remove), swap, move, batch remove, retain, and more.
- **Builder API** — chain operations to build transactions with a fluent interface.
- **No `unsafe`** — 100% safe Rust.

## Key type requirements

The map key type `K` must implement:

| Trait     | Required by                                                        |
|-----------|--------------------------------------------------------------------|
| `Hash`    | All operations — used to determine the shard for a key.            |
| `Eq`      | All operations — used for hash-map lookups within each shard.      |
| `Clone`   | Required by many operations: `insert_with`, `modify`, `map`, `swap_value`, `remove`, `retain_only`, guard conditions, and the `get` finisher. |

The value type `V` has no trait bounds by default. Operations that create values (e.g., `modify_or_default`) require `V: Default`.

## Usage

Add `txmap` to your `Cargo.toml`:

```toml
[dependencies]
txmap = "0.1.0"
```

### Creating a `TxMap`

```rust
use txmap::prelude::*;

// Choose a shard count — powers of two from 8 to 128
let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
```

Larger shard counts reduce lock contention at the cost of slightly more memory. Choose the smallest count that gives adequate concurrency for your workload.

### Basic operations

```rust
use txmap::prelude::*;

let map = TxMap::new(ShardCount::_8);

// Insert
map.insert("alice".to_string(), 100u64);

// Get a value (with a transform closure)
let balance = map.get_with(&"alice".to_string(), |v| *v);
assert_eq!(balance, Some(100));

// Remove
let old = map.remove(&"alice".to_string());
assert_eq!(old, Some(100));

// Length / empty
assert!(map.is_empty());
assert_eq!(map.len(), 0);

// Clear all entries
map.clear();
```

### Transactions

Transactions group multiple operations into an atomic unit. They are built using a fluent builder API.

#### Simple transaction (no result)

```rust
use txmap::prelude::*;

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);

db.insert("alice".to_string(), 100);

// Transfer 50 from alice to bob in one transaction
db.transaction()
    .modify("alice".to_string(), |_name, balance| {
        *balance -= 50;
    })
    .modify_or_default("bob".to_string(), |_name, balance| {
        *balance += 50;
    })
    .into_transaction()
    .execute();

assert_eq!(db.get_with(&"alice".to_string(), |v| *v), Some(50));
assert_eq!(db.get_with(&"bob".to_string(), |v| *v), Some(50));
```

#### Transaction with guards (preconditions)

```rust
use txmap::prelude::*;

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);
db.insert("alice".to_string(), 100);

let tx = db
    .transaction()
    .require(
        "Alice has sufficient funds",
        ["alice".to_string()],
        |[alice_balance]| alice_balance.is_some_and(|b| *b >= 50),
    )
    .modify("alice".to_string(), |_name, balance| {
        *balance -= 50;
    })
    .modify_or_default("bob".to_string(), |_name, balance| {
        *balance += 50;
    })
    .into_transaction();

match tx.execute() {
    TxResult::Completed(()) => println!("Transfer succeeded"),
    TxResult::RequirementNotMet(_, name) => {
        println!("Transaction rejected: {name}")
    }
}
```

If any guard condition fails, the transaction is **not executed** — no locks are held, no mutations occur — and `TxResult::RequirementNotMet` is returned.

#### Transaction returning a value

```rust
use txmap::prelude::*;

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);
db.insert("alice".to_string(), 100);

let new_balance = db
    .transaction()
    .modify("alice".to_string(), |_name, balance| {
        *balance += 50;
    })
    .get("alice".to_string(), |_name, balance| *balance)
    .into_transaction()
    .execute();

assert_eq!(new_balance, TxResult::Completed(Some(150)));
```

#### Transaction returning multiple values

```rust
use txmap::prelude::*;

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);
db.insert("alice".to_string(), 100);
db.insert("bob".to_string(), 200);

let balances = db
    .transaction()
    .get_all(
        vec!["alice".to_string(), "bob".to_string()],
        |_name, balance| *balance,
    )
    .into_transaction()
    .execute();

assert_eq!(balances, TxResult::Completed(vec![Some(100), Some(200)]));
```

### Parameterized transactions

Parameterized transactions let you pass a parameter struct to all closures. This is useful for reusable transaction logic.

```rust
use txmap::prelude::*;

#[derive(Default)]
struct Transfer {
    amount: u64,
}

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);
db.insert("alice".to_string(), 200);

let transfer_tx = db
    .transaction()
    .with_param::<Transfer>()
    .require(
        "Alice has sufficient funds",
        ["alice".to_string()],
        |[balance], params| balance.is_some_and(|b| *b >= params.amount),
    )
    .modify("alice".to_string(), |_name, balance, params| {
        *balance -= params.amount;
    })
    .modify_or_default("bob".to_string(), |_name, balance, params| {
        *balance += params.amount;
    })
    .get("alice".to_string(), |_name, balance| *balance)
    .into_transaction();

// Execute with different parameters
let result1 = transfer_tx.execute(&Transfer { amount: 50 });
assert_eq!(result1, TxResult::Completed(Some(150)));

let result2 = transfer_tx.execute(&Transfer { amount: 30 });
assert_eq!(result2, TxResult::Completed(Some(120)));
```

### Operation reference

| Builder method                          | Description                                                       | Requires `K: Clone` | Requires `V: Default` |
|-----------------------------------------|-------------------------------------------------------------------|----------------------|------------------------|
| `modify(key, \|k, v\| ...)`             | Mutate an existing value in-place. Does nothing if key absent.     |                      |                        |
| `modify_or_default(key, \|k, v\| ...)`  | Like `modify` but inserts `V::default()` if key absent.           | ✓                    | ✓                      |
| `modify_or_insert_with(key, \|k, v\| ..., \|k\| -> V)` | Like `modify` but inserts via a generator if key absent. | ✓                    |                        |
| `map(key, \|k, opt_v\| -> Option<V>)`   | Insert, update, or remove a single entry. Return `Some(v)` to set, `None` to delete. | ✓ | |
| `insert_with(key, \|k\| -> V)`          | Insert a value, generated from the key.                           | ✓                    |                        |
| `insert_default(key)`                   | Insert `V::default()` for the key.                                | ✓                    | ✓                      |
| `swap_value(a, b)`                      | Swap the values of two keys.                                      | ✓                    |                        |
| `move_value(from, to)`                  | Move the value from one key to another (removes `from`).          | ✓                    |                        |
| `remove(keys)`                          | Remove a batch of keys.                                           | ✓                    |                        |
| `remove_where(keys, \|k, v\| -> bool)`  | Remove keys that satisfy a condition.                             | ✓                    |                        |
| `retain_only(keys)`                     | Remove all entries **not** in the given set.                      | ✓                    |                        |
| `retain_where(keys, \|k, v\| -> bool)`  | Remove entries not in the set unless they satisfy a condition.    | ✓                    |                        |
| `clear()`                               | Remove all entries.                                               |                      |                        |
| `remove_if(\|k, v\| -> bool)`           | Remove all entries satisfying a condition.                        |                      |                        |
| `retain(\|k, v\| -> bool)`              | Keep only entries satisfying a condition.                         |                      |                        |
| `modify_peek(key, peek_keys, \|k, v, [peek_vals]\| ...)` | Modify a value while reading (peeking) other keys. | ✓ (for peek keys)    |                        |
| `map_peek(key, peek_keys, \|k, opt_v, [peek_vals]\| -> Option<V>)` | Map a value while peeking others. | ✓ (for peek keys) | |

**Finisher methods** (called last in the builder chain):

| Method              | Description                                    | Transaction result type          |
|---------------------|------------------------------------------------|----------------------------------|
| `.get(key, \|k, v\| -> R)` | Read a single value after mutations.     | `TxResult<Option<R>>`            |
| `.get_all(keys, \|k, v\| -> R)` | Read multiple values after mutations. | `TxResult<Vec<Option<R>>>`      |
| *(none — call `.into_transaction().execute()`)* | Execute with no return value. | `TxResult<()>` |

## `TxResult`

All transactions return `TxResult<T>`:

```rust
pub enum TxResult<T> {
    Completed(T),
    RequirementNotMet(usize, String),
}
```

- `Completed(result)` — the transaction was executed successfully.
- `RequirementNotMet(index, name)` — a guard condition failed; the transaction was aborted. The `index` indicates which guard failed, and `name` is the user-supplied description.

## Comparison with `std::sync::RwLock<HashMap>`

| Aspect               | `RwLock<HashMap>`                           | `TxMap`                                      |
|----------------------|----------------------------------------------|----------------------------------------------|
| Concurrency          | Single lock — serializes all access            | Sharded locks — concurrent access to different shards |
| Atomic multi-key ops | Manual — risk of deadlock                     | Built-in — careful lock ordering             |
| Transactional guards | Manual                                        | Declarative `require()`                      |
| Parameterized ops    | Not supported                                 | Built-in `with_param::<P>()`                 |
| Memory               | Single `HashMap`                              | Multiple `HashMap`s (one per shard)          |

## License

Licensed under the [MIT License](LICENSE).
