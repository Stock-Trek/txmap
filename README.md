# TxMap

[![crates.io](https://img.shields.io/crates/v/txmap)](https://crates.io/crates/txmap)
[![docs.rs](https://img.shields.io/docsrs/txmap)](https://docs.rs/txmap)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A concurrent transactional hash map for Rust with fine-grained user-defined locking, internal mutability for easy sharing, and composable transactions.

## Features

- [**Flexible sharding**](#lock-policy) Choose the number of shards (between 8 and 128). Decide how they're locked (Mutex, RwLock or bring your own)
- [**Transactions**](#transactions) Atomic, composable batches of modifications
- [**Guards/conditions**](#transaction-with-guards-preconditions) Declarative preconditions that must hold before a transaction runs
- [**Parameterized transactions**](#parameterized-transactions) Optionally define a parameter type to pass into transaction closures
- [**Builder API**](#transaction-operations) Chain operations to build transactions with a fluent interface

## License

Licensed under the [MIT License](LICENSE).

## Usage

Add `txmap` to your `Cargo.toml`:

```toml
[dependencies]
txmap = "1.0.0"
```

### Creating a `TxMap`

```rust
use txmap::prelude::*;

// Shard counts available are [8, 16, 32, 64, 128]
let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
```

### Lock Policy

The lock policy can be set using the `with_lock_policy` constructor.
Two policies are provided: [MutexLockPolicy](./src/locks/mutex_policy.rs) and [RwLockPolicy](./src/locks/rwlock_policy.rs). The default is MutexLockPolicy.
You can also implement your own policy by implementing [LockPolicy](./src/locks/lock_policy.rs).

```rust
// Creating a TxMap with a lock policy
let map = TxMap::with_lock_policy::<MyLockPolicy>(ShardCount::_8);
```

### Key type requirements

The key type `K` must implement `Hash` and `Eq`. Some functions require `Clone`.

The value type `V` has no trait bounds by default. Functions that create default values (e.g., `insert_default`) require `V: Default`.

### Transactions

Transactions group multiple operations into an atomic unit. They are built using a fluent builder API. Use `.into_transaction()` to produce a reusable transaction, then `.execute()` to run it. They can be run as many times as you want within the lifetime of the TxMap it was created from.

#### Simple transaction

```rust
use txmap::prelude::*;

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);

db.insert("alice".into(), 100);
db.insert("bob".into(), 0);

// Transfer 50 from alice to bob in one atomic transaction
let result = db.transaction()
    .modify("alice".into(), |_name, balance| {
        *balance -= 50;
    })
    .modify("bob".into(), |_name, balance| {
        *balance += 50;
    })
    .get_copied(vec!["alice".into(), "bob".into()])
    .into_transaction()
    .execute();

match result {
    // every .execute() call return a TxResult
    case TxResult::Completed(balances) => {
        assert_eq!(balances, vec![Some(50), Some(50)]);
    },
    _ => {}
}
```

#### Transaction with guards (preconditions)

Perhaps Alice doesn't have enough funds to make a transfer and you need to prevent a transfer happening.
Guards can be used to veto a transaction if they fail, in which case `TxResult::RequirementNotMet` is returned.

```rust
use txmap::prelude::*;

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);
db.insert("alice".into(), 100);
db.insert("bob".into(), 0);

let result = db
    .transaction()
    // Add all your requirements up front
    // Requirements cannot be added after modifications
    .require(
        "Alice has sufficient funds",
        ["alice".into()],
        |[balance]| balance.is_some_and(|b| *b >= 250),
    )
    .require(
        "Bob has an account",
        ["bob".into()],
        |[balance]| balance.is_some(),
    )
    .modify("alice".into(), |_name, balance| {
        *balance -= 250;
    })
    .modify("bob".into(), |_name, balance| {
        *balance += 250;
    })
    .get_copied(vec!["alice".into(), "bob".into()])
    .into_transaction()
    .execute();

match result {
    case TxResult::RequirementNotMet(guard_idx, guard_name) => {
        assert_eq!(guard_idx, 0);
        assert_eq!(guard_name, "Alice has sufficient funds");
    },
    _ => {}
}
```

#### Transaction returning a value

As seen already, a completed transaction can optionally return a final value in the result. There are 6 functions like this that can be used

| Final function                           | Description                | Additional bounds required |
|------------------------------------------|----------------------------|----------------------------|
| *none (default)*                         | Returns unit type ()       |                            |
| `get_copied(key)`                        | Copies a value             | `V: Copy`                  |
| `get_all_copied(keys)`                   | Copies a vec of values     | `V: Copy`                  |
| `get_cloned(key)`                        | Clones a value             | `V: Clone`                 |
| `get_all_cloned(keys)`                   | Clones a vec of values     | `V: Clone`                 |
| `get_with(key, \|&K, &V\| { ... })`      | Transforms a value         |                            |
| `get_all_with(keys, \|&K, &V\| { ... })` | Transforms a vec of values |                            |

### Parameterized transactions

A transaction may need to be called with different parameters.

The function `with_param::<MyParams>()` lets you do this by parameterizing the transaction.

```rust
use txmap::prelude::*;

#[derive(Default)]
struct Transfer {
    amount: u64,
}

let db: TxMap<String, u64> = TxMap::new(ShardCount::_8);
db.insert("alice".into(), 200);
db.insert("bob".into(), 0);

let transfer = db
    .transaction()
    // Parameter type is set before requirements
    .with_param::<Transfer>()
    .require(
        "Alice has sufficient funds",
        ["alice".into()],
        |[balance], params| balance.is_some_and(|b| *b >= params.amount),
    )
    .modify("alice".into(), |_name, balance, params| {
        *balance -= params.amount;
    })
    .modify("bob".into(), |_name, balance, params| {
        *balance += params.amount;
    })
    .get_all_copied(["alice".into(), "bob".into()])
    .into_transaction();

// Execute with different parameters
let result1 = transfer.execute(&Transfer { amount: 50 });
assert_eq!(result1, TxResult::Completed(vec![Some(150), Some(50)]));

let result2 = transfer.execute(&Transfer { amount: 30 });
assert_eq!(result2, TxResult::Completed(vec![Some(120), Some(80)]));
```

### Transaction operations

Transactions are built with a fluent interface, many operations can be chained together atomically into a single transaction.
Available transaction operations are as follows

| Transaction operation      | Description                                                                  | Additional required bounds |
|----------------------------|------------------------------------------------------------------------------|----------------------------|
| `insert_default`           | Insert `V::default()` for the key.                                           | `K: Clone` `V: Default`    |
| `insert_default_if_absent` | Insert `V::default()` for the key, only if the key is absent.                | `K: Clone` `V: Default`    |
| `insert_with`              | Insert a value generated from the key.                                       | `K: Clone`                 |
| `insert_with_if_absent`    | Insert a value generated from the key, only if the key is absent.            | `K: Clone`                 |
| `modify`                   | Mutate an existing value in-place. Does nothing if key absent.               |                            |
| `modify_peek`              | Like `modify` while peeking at other values.                                 |                            |
| `update`                   | Update a single entry. Return `Some(v)` to insert/replace, `None` to remove. | `K: Clone`                 |
| `update_peek`              | Like `update` while peeking at other values.                                 | `K: Clone`                 |
| `move_value`               | Remove a value from one key and insert it with another key.                  | `K: Clone`                 |
| `swap_value`               | Swap the values of two keys.                                                 | `K: Clone`                 |
| `remove`                   | Remove the given keys.                                                       |                            |
| `remove_where`             | Remove the given keys which also satisfy a condition.                        |                            |

## TxMap operations

All transaction operations are also available on TxMap.
In addition there are some operations that require locking the entire map which are only available on TxMap. These are as follows

| TxMap operation | Description                                                |
|-----------------|------------------------------------------------------------|
| `clear`         | Removes all entries                                        |
| `remove_if`     | Removes any entry which satisfies a condition              |
| `retain_only`   | Retains only keys specified                                |
| `retain_where`  | Retains only keys specified which also satisfy a condition |
| `retain`        | Retains any entry which satisfies a condition              |
