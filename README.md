# TxMap

[![crates.io](https://img.shields.io/crates/v/txmap)](https://crates.io/crates/txmap)
[![docs.rs](https://img.shields.io/docsrs/txmap)](https://docs.rs/txmap)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A concurrent transactional hash map for Rust with fine-grained user-defined locking, internal mutability for easy sharing, and composable transactions.

## Features

- [**Flexible sharding**](#lock-policy) Choose the number of shards (between 8 and 128). Decide how they're locked (Mutex, RwLock or bring your own)
- [**Immediate Transactions**](#immediate-transactions) Immediately execute an atomic, composable batch of modifications
- [**Parameterized Transactions**](#parameterized-transactions) Create re-usable transactions for faster parameterized execution
- [**Guards/conditions**](#transaction-with-guards-preconditions) Declarative preconditions that must hold before a transaction runs
- [**Fluent API**](#transaction-operations) Chain operations to build or execute transactions with a fluent interface
- **Optional serde support** Use the `serde` feature to enable it

## License

Licensed under the [MIT License](LICENSE).

## Usage

Add `txmap` to your `Cargo.toml`:

```toml
[dependencies]
txmap = "2.1.1"
```

### Creating a `TxMap`

```rust
use txmap::prelude::*;

// Shard counts available are [8, 16, 32, 64, 128]
let map: TxMap<String, u64> = TxMap::new(Shards::_8);
```

### Lock Policy

The lock policy can be set using the `with_lock_policy` constructor.
Two policies are provided: [MutexLockPolicy](./src/locks/mutex_policy.rs) and [RwLockPolicy](./src/locks/rwlock_policy.rs). The default is MutexLockPolicy.
You can also use your own policy by implementing [LockPolicy](./src/locks/lock_policy.rs).

```rust
// Creating a TxMap with a lock policy
let map = TxMap::with_lock_policy::<MyLockPolicy>(Shards::_8);
```

### Key type requirements

The key type `K` must implement `Hash` and `Eq`. Some functions require `Clone`.

The value type `V` has no trait bounds by default. Functions that create default values (e.g., `insert_default`) require `V: Default`.

### Immediate Transactions

You may need to run a one-off transaction, for these cases use `map.immediate_tx()`.

```rust
use txmap::prelude::*;

// Create a state struct for the transaction, it must implement Default
#[derive(Default)]
struct TransferState {
    new_from: u64,
    new_to: u64,
}

let db: TxMap<String, u64> = TxMap::new(Shards::_8);

db.insert("alice".into(), 100);
db.insert("bob".into(), 0);

// Use the state struct as the generic type
// The transaction will create an instance to use as the mutable state and return it wrapped in a TxResult
let result = db.immediate_tx::<TransferState>()
    // Transfer 50 from alice to bob in one atomic transaction
    .modify("alice".into(), |_name, balance, state| {
        *balance -= 50;
        state.new_from = *balance;
    })
    .modify("bob".into(), |_name, balance, state| {
        *balance += 50;
        state.new_to = *balance;
    })
    .execute();

assert!(matches!(result, TxResult::Completed(TransferState { new_from: 50, new_to: 50 })));
```

### Parameterized transactions

Some transaction might need to be run many times, or with different parameters. For these cases create a prepared transaction with `map.prepared_tx()`.
Prepared transactions need a transaction schema, created via the macro `tx_schema`.

```rust
use txmap::prelude::*;

// The example below will create 4 types to use in the transaction
// 1. Transfer: Contains constants for SCHEMA and all key handles used in the transaction, (from and to)
// 2. TransferKeys: Used to pass in the actual keys for each execution
// 3. TransferParams: Used to pass in the real parameters for each execution
// 4. TransferState: Used by the transaction to store state and is returned as part of the final result

tx_schema! {
    Transfer,               // transaction name
    keys: [from, to],       // define all key handles the transaction will use
    params: {               // parameters for the transaction
        amount: u64,
        commission: u64,
    },
    state: {                // define fields for the local working space, each execution will create a new one
        total_cost: u64,
        total_received: u64,
        commission_paid: u64,
    }
}

let db: TxMap<String, u64> = TxMap::new(Shards::_8);
db.insert("alice".into(), 200);
db.insert("bob".into(), 0);

let transfer_tx = db
    .prepared_tx(&Transfer::SCHEMA) // pass in the SCHEMA constant
    // prepared transactions also pass in your parameters to all closures
    .modify(
        Transfer::from, // use the key handles available, these are populated per execution
        |_name, balance, params, _state| {
            // changes are safe to make separately as the whole transaction is atomic
            *balance -= params.amount;
        }
    )
    .modify(
        Transfer::to,
        |_name, balance, params, state| {
            let received = params.amount * (100 - params.commission);
            state.total_cost = params.amount;
            state.total_received = received;
            state.commission_paid = params.amount - received;
            *balance += received;
        }
    )
    .into_transaction();

    // Execute with different parameters
    // Use the ...Keys and ...Params structs created by the `tx_schema` macro
    let result1 = transfer_tx.execute(
        TransferKeys {
            from: "alice".into(),
            to: "bob".into(),
        },
        TransferParams {
            amount: 50,
            commission: 0,
        },
    );
    assert_eq!(
        result1,
        TxResult::Completed(TransferState {
            total_cost: 100,
            total_received: 100,
            commission_paid: 0
        })
    );

    let result2 = transfer_tx.execute(
        TransferKeys {
            from: "alice".into(),
            to: "bob".into(),
        },
        TransferParams {
            amount: 50,
            commission: 10,
        },
    );
    assert_eq!(
        result2,
        TxResult::Completed(TransferState {
            total_cost: 50,
            total_received: 45,
            commission_paid: 5
        })
    );
```

### Transaction with guards (preconditions)

Perhaps Alice doesn't have enough funds to make a transfer and you need to prevent a transfer if it would cause a negative balance.
Guards can be used to veto a transaction if they fail, in which case `TxResult::RequirementNotMet` is returned.

```rust
use txmap::prelude::*;

#[derive(Default)]
struct TransferResult {
    new_from: Option<u64>,
    new_to: Option<u64>,
}

let db: TxMap<String, u64> = TxMap::new(Shards::_8);
db.insert("alice".into(), 100);
db.insert("bob".into(), 0);

let result = db
    .immediate_tx::<TransferResult>()
    // Add all your requirements up front
    // Requirements cannot be added after modifications
    .require(
        "Alice has sufficient funds",
        "alice".into(),
        |_name, balance, _state| balance.is_some_and(|b| *b >= 250),
    )
    .require(
        "Bob has an account",
        "bob".into(),
        |_name, balance, _state| balance.is_some(),
    )
    .modify("alice".into(), |_name, balance, state| {
        *balance -= 250;
        state.new_from = Some(*balance);
    })
    .modify("bob".into(), |_name, balance, state| {
        *balance += 250;
        state.new_to = Some(*balance);
    })
    .execute();

match result {
    TxResult::RequirementNotMet(guard_idx, guard_name, state) => {
        assert_eq!(guard_idx, 0);
        assert_eq!(guard_name, "Alice has sufficient funds");
        assert!(
            matches!(
                state,
                TransferResult {
                    new_from: None,
                    new_to: None
                }
            )
        );
    },
    _ => {}
}
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
| `move_value`               | Remove a value from one key and insert it with another key.                  | `K: Clone`                 |
| `remove`                   | Remove the given key.                                                        |                            |
| `remove_where`             | Remove the given key if it also satisfies a condition.                       |                            |
| `swap_value`               | Swap the values of two keys.                                                 | `K: Clone`                 |
| `update`                   | Update a single entry. Return `Some(v)` to insert/replace, `None` to remove. | `K: Clone`                 |

### TxMap operations

All transaction operations are also available on TxMap.
In addition there are some operations that require locking the entire map which are only available on TxMap. These are as follows

| TxMap operation | Description                                                |
|-----------------|------------------------------------------------------------|
| `clear`         | Removes all entries                                        |
| `remove_if`     | Removes any entry which satisfies a condition              |
| `retain_only`   | Retains only keys specified                                |
| `retain_where`  | Retains only keys specified which also satisfy a condition |
| `retain`        | Retains any entry which satisfies a condition              |
