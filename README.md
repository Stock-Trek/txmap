# TxMap

[![crates.io](https://img.shields.io/crates/v/txmap)](https://crates.io/crates/txmap)
[![docs.rs](https://img.shields.io/docsrs/txmap)](https://docs.rs/txmap)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A concurrent transactional hash map for Rust with fine-grained locking and internal mutability.

`TxMap` partitions stored key-value pairs across multiple shards, each protected by its own `parking_lot::Mutex` and backed by its own `hashbrown::HashMap`. Read and write operations acquire locks only on the shards they need, maximizing concurrency. Transactional operations group multiple operations into atomic units, and support parameterized closures.

## Features

- **Concurrent access** Fine-grained shard-level locking; operations lock only the shards they touch
- **Transactions** Atomic, composable batches of modifications
- **Optional parameterized transactions** Optionally define a parameter type to pass into transaction closures
- **Guards/conditions** Declarative preconditions that must hold before a transaction runs
- **Flexible operations** Modify, map, insert, remove, swap, move, retain, and more
- **Builder API** Chain operations to build transactions with a fluent interface
- **No `unsafe`** 100% safe Rust

## License

Licensed under the [MIT License](LICENSE).

## Usage

Add `txmap` to your `Cargo.toml`:

```toml
[dependencies]
txmap = "0.1.0"
```

### Creating a `TxMap`

```rust
use txmap::prelude::*;

// Choose a shard count, a power of two between 8 to 128 inclusive
let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
```

Larger shard counts reduce lock contention at the cost of slightly more memory. Choose the smallest count that gives adequate concurrency for your workload.

### Key type requirements

The map key type `K` must implement `Hash` and `Eq`. Some functions also require `Clone`.
The value type `V` has no trait bounds by default. Operations that create default values (e.g., `modify_or_default`) require `V: Default`.

### Basic operations

```rust
use txmap::prelude::*;

let map = TxMap::new(ShardCount::_8);

// Insert
map.insert("alice".to_string(), 100u64);

// Get a value with a transform closure
let balance = map.get_with(&"alice".to_string(), |v| *v);
assert_eq!(balance, Some(100));

// Copy a value directly (requires V: Copy)
let copied = map.get_copied(&"alice".to_string());
assert_eq!(copied, Some(100));

// Remove
let old = map.remove(&"alice".to_string());
assert_eq!(old, Some(100));

// Clear all entries
map.clear();

// Length / empty
assert!(map.is_empty());
assert_eq!(map.len(), 0);

// Fold over all entries
map.insert("a".to_string(), 10);
map.insert("b".to_string(), 20);
let sum = map.fold(0u64, |_k, v| Some(*v), |acc, v| acc + v);
assert_eq!(sum, 30);
```

### Transactions

Transactions group multiple operations into an atomic unit. They are built using a fluent builder API. Use `.into_transaction()` to produce a reusable transaction, then `.execute()` to run it.

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

Guards are checked before any mutations take place. If any guard fails, the transaction is **not executed** — no locks are held, no mutations occur — and `TxResult::RequirementNotMet` is returned.

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

#### Transaction returning a value

Use `.get()` or `.get_copied()` before `.into_transaction()` to return a value from the transaction.

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

let transfer_alice_to_bob_tx = db
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
    .get_all(["alice".to_string(), "bob".to_string()], |_name, balance| *balance)
    .into_transaction();

// Execute with different parameters
let result1 = transfer_alice_to_bob_tx.execute(&Transfer { amount: 50 });
assert_eq!(result1, TxResult::Completed([Some(150), Some(50)]));

let result2 = transfer_alice_to_bob_tx.execute(&Transfer { amount: 30 });
assert_eq!(result2, TxResult::Completed([Some(120), Some(80)]));
```

### Finite state machine example

Use `update` to implement state transitions that return `Some(new_state)` to update or `None` to delete.

```rust
use txmap::prelude::*;

#[derive(Clone, PartialEq)]
enum OrderState { Pending, Shipped, Delivered }

let orders: TxMap<String, OrderState> = TxMap::new(ShardCount::_8);
orders.insert("order-1".into(), OrderState::Pending);

// Transition order-1 from Pending to Shipped
let result = orders
    .transaction()
    .update("order-1".into(), |_id, state| {
        state.and_then(|s| match s {
            OrderState::Pending => Some(OrderState::Shipped),
            _ => None, // reject transition, deleting the entry
        })
    })
    .get_cloned("order-1".into())
    .into_transaction()
    .execute();

assert_eq!(result, TxResult::Completed(Some(OrderState::Shipped)));
```

### Swap and move operations

Atomically swap values between two keys, or move a value from one key to another.

```rust
use txmap::prelude::*;

let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
map.insert("a".to_string(), 10);
map.insert("b".to_string(), 20);

// Swap values
let result = map
    .transaction()
    .swap_value("a".to_string(), "b".to_string())
    .get_all_copied(["a".to_string(), "b".to_string()])
    .into_transaction()
    .execute();
assert_eq!(result, TxResult::Completed(vec![Some(20), Some(10)]));

// Move value (from "a" to "b", leaving "a" empty)
let result = map
    .transaction()
    .move_value("a".to_string(), "b".to_string())
    .get_all_copied(["a".to_string(), "b".to_string()])
    .into_transaction()
    .execute();
assert_eq!(result, TxResult::Completed(vec![None, Some(20)]));
```

### Batch operations

Conditionally remove or retain entries in bulk.

```rust
use txmap::prelude::*;

let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
map.insert("alice".into(), 1);
map.insert("bob".into(), 2);
map.insert("chuck".into(), 3);

// Remove entries with values > 1
map.transaction()
    .remove_if(|_k, v| *v > 1)
    .into_transaction()
    .execute();
assert_eq!(map.len(), 1); // only alice remains

// Retain only specific keys
map.insert("bob".into(), 2);
map.transaction()
    .retain_only(["alice".into(), "bob".into()])
    .into_transaction()
    .execute();
assert_eq!(map.len(), 2);
```

### Operation reference

| Builder method               | Description                                                                       | Additional required bounds |
|------------------------------|-----------------------------------------------------------------------------------|----------------------------|
| `insert_default`             | Insert `V::default()` for the key.                                                | `K: Clone` `V: Default`    |
| `insert_default_if_absent`   | Insert `V::default()` for the key, only if the key is absent.                     | `K: Clone` `V: Default`    |
| `insert_with`                | Insert a value generated from the key.                                            | `K: Clone`                 |
| `insert_with_if_absent`      | Insert a value generated from the key, only if the key is absent.                 | `K: Clone`                 |
| `modify`                     | Mutate an existing value in-place. Does nothing if key absent.                    |                            |
| `modify_peek`                | Like `modify` while peeking at other values.                                      | `K: Clone`                 |
| `update`                     | Update a single entry. Return `Some(v)` to insert/replace, `None` to delete.      | `K: Clone`                 |
| `update_peek`                | Like `update` while peeking at other values.                                      | `K: Clone`                 |
|                              |                                                                                   |                            |
| `move_value`                 | Remove a value from one key and insert it with another key.                       | `K: Clone`                 |
| `swap_value`                 | Swap the values of two keys.                                                      | `K: Clone`                 |
|                              |                                                                                   |                            |
| `remove`                     | Remove the given keys.                                                            |                            |
| `remove_where`               | Remove the given keys which also satisfy a condition.                             |                            |
| `retain_only`                | Retain only the given keys.                                                       |                            |
| `retain_where`               | Retain only the given keys which also satisfy a condition.                        |                            |
|                              |                                                                                   |                            |
| `clear`                      | Remove all entries.                                                               |                            |
| `remove_if`                  | Remove any entries which satisfy a condition.                                     |                            |
| `retain`                     | Retain only the entries which satisfy a condition.                                |                            |

### Finisher methods

Up to one of these is called before `.into_transaction()` to define what the transaction should return.

| Method                                      | Description                                              | Transaction result type    |
|---------------------------------------------|----------------------------------------------------------|----------------------------|
| *(none - default)*                          | Execute with no return value.                            | `TxResult<()>`             |
| `get_copied(key)`                           | Copy a single value. Requires `V: Copy`                  | `TxResult<Option<V>>`      |
| `get_all_copied(keys)`                      | Copy an array of values. Requires `V: Copy`              | `TxResult<Vec<Option<V>>>` |
| `get_cloned(key)`                           | Clone a single value. Requires `V: Clone`                | `TxResult<Option<V>>`      |
| `get_all_cloned(keys)`                      | Clone an array of values. Requires `V: Clone`            | `TxResult<Vec<Option<V>>>` |
| `get(key, \|k, v[, params]\| { ... })`      | Read a single value and apply a transformation to it.    | `TxResult<Option<R>>`      |
| `get_all(keys, \|k, v[, params]\| { ... })` | Read multiple values and apply a transformation to them. | `TxResult<Vec<Option<R>>>` |

To create the final transaction call `into_transaction()`. This will produce a re-useable transaction that can be executed as many times as you want within the lifetime of its `TxMap`.

### `TxResult`

All transactions return `TxResult<T>`:

```rust
pub enum TxResult<T> {
    Completed(T),
    RequirementNotMet(usize, String),
}
```

- `Completed(result)` The transaction was executed successfully.
- `RequirementNotMet(index, name)` A guard condition failed; the transaction was aborted. The `index` indicates which guard failed, and `name` is the user-supplied description.

## Operation appendix

| Operation                                                                                      |
|------------------------------------------------------------------------------------------------|
| `insert_default(key)`                                                                          |
| `insert_default_if_absent(key)`                                                                |
| `insert_with(key, \|k[, params]\| { new_value } )`                                             |
| `insert_with_if_absent(key, \|k[, params]\| { new_value } )`                                   |
| `modify(key, \|k, mut v[, params]\|)`                                                          |
| `modify_peek(key, peek_keys, \|k, mut v, pks[, params]\|)`                                     |
| `update(key, \|k, v_opt[, params]\| { new_value_opt })`                                        |
| `update_peek(key, transform, peek_keys)`                                                       |
|                                                                                                |
| `move_value(from, to)`                                                                         |
| `swap_value(a, b)`                                                                             |
|                                                                                                |
| `remove(keys)`                                                                                 |
| `remove_where(keys, \|k, v[, params]\| { remove })`                                            |
| `retain_only(keys)`                                                                            |
| `retain_where(keys, \|k, v[, params]\| { remove })`                                            |
|                                                                                                |
| `clear()`                                                                                      |
| `remove_if(\|k, v[, params]\| { remove })`                                                     |
| `retain(\|k, v[, params]\| { remove })`                                                        |
