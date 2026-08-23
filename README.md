# TxMap

[![crates.io](https://img.shields.io/crates/v/txmap)](https://crates.io/crates/txmap)
[![docs.rs](https://img.shields.io/docsrs/txmap)](https://docs.rs/txmap)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A concurrent transactional hash map for Rust with fine-grained user-defined locking, internal mutability for easy sharing, and composable transactions.

## Features

- [**Proven performance**](https://github.com/Stock-Trek/map-benchmarks) One of the fastest concurrent maps available
- [**Customizable**](#creating-a-txmap) Choose the number of shards, shard locking policy (Mutex, RwLock or bring your own) and capacity
- [**Immediate Transactions**](#immediate-transactions) Immediately execute an atomic, composable batch of modifications
- [**Parameterized Transactions**](#parameterized-transactions) Create re-usable transactions for faster parameterized execution
- [**Guards/conditions**](#transaction-with-guards-preconditions) Declarative preconditions that must hold before a transaction runs
- [**Fluent API**](#transaction-operations) Chain operations to build or execute transactions with a fluent interface
- **Optional serde support** Use the `serde` feature to enable it
- **Optional rayon support** Use the `rayon` feature for parallel iterators (`par_iter`, `par_keys`, `par_values`)
- **Rapidhash hasher by default** The `rapidhash` feature (enabled by default) uses the [rapidhash](https://crates.io/crates/rapidhash) hasher for improved performance. Disable default features and opt out with `default-features = false` in your `Cargo.toml` to fall back to the standard library's `RandomState` (SipHash).

## License

Licensed under the [MIT License](LICENSE).

## Usage

Add `txmap` to your `Cargo.toml`:

```toml
[dependencies]
txmap = "3.5.1"
```

### Creating a `TxMap`

```rust
use txmap::prelude::*;

let map = TxMap::new();
```

This creates a map using the default options. To customise these options you can use a map builder which allows configuring them.

| Option           | Default                      |
|------------------|------------------------------|
| Shard count      | 32                           |
| Lock policy      | MutexPolicy                  |
| Hasher           | rapidhash::fast::RandomState |
| Initial capacity | 0                            |

Two shard locking policies are provided: [MutexPolicy](./src/lock_policies/mutex_policy.rs) (default) and [RwLockPolicy](./src/lock_policies/rwlock_policy.rs).
You can also use your own policy by implementing [LockPolicy](./src/lock_policies/lock_policy.rs).

```rust
// Creating a TxMap via a builder
let map = TxMapBuilder::default()
            .with_shards(Shards::_8)
            .with_lock_policy::<RwLockPolicy>()
            .with_capacity(10_000)
            .build();
```

### Key type requirements

The key type `K` should implement `Hash` and `Eq` for it to be useful outside of iteration. There are also a few additional functions that require `K: Clone`.

The value type `V` has no required trait bounds apart from a few specific cases such as when cloning the map or a value that require `V: Clone`.

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

let db: TxMap<String, u64> = TxMap::new();

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

assert!(matches!(result, TxResult::Completed { state: TransferState { new_from: 50, new_to: 50 } }));
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

let db: TxMap<String, u64> = TxMap::new();
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
        TxResult::Completed {
            state: TransferState {
                total_cost: 100,
                total_received: 100,
                commission_paid: 0
            }
        }
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
        TxResult::Completed {
            state: TransferState {
                total_cost: 50,
                total_received: 45,
                commission_paid: 5
            }
        }
    );
```

In addition to the 4 types above, `tx_schema!` also generates specialised
prepared transaction types and an entry function that bake the schema in, so
prepared transactions are easy to store for later use:

```rust
// 5. TransferPreparedTransaction: a specialised PreparedTransaction that only
//    needs the map's key type as a generic parameter when stored.
// 6. TransferPreparedTxBuilder: a specialised PreparedTxBuilder (the lock
//    policy and hasher default to MutexPolicy / DefaultBuildHasher).
// 7. transfer_prepared_tx: a specialised prepared_tx function that infers
//    every type from the map, so no generic parameters need to be written.

struct App<'tx> {
    transfer: TransferPreparedTransaction<'tx, String>, // one generic parameter
}

let app = App {
    transfer: transfer_prepared_tx(&db) // zero generic parameters
        .modify(Transfer::from, |_name, balance, params, _state| {
            *balance -= params.amount;
        })
        .modify(Transfer::to, |_name, balance, params, _state| {
            *balance += params.amount;
        })
        .into_transaction(),
};
```

The generic `PreparedTransaction` is also storable with a single schema
generic: `PreparedTransaction<'tx, Transfer<String>>`.

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

let db: TxMap<String, u64> = TxMap::new();
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
    TxResult::RequirementNotMet { index, requirement, state } => {
        assert_eq!(index, 0);
        assert_eq!(requirement, "Alice has sufficient funds");
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

| Transaction operation      | Description                                                                  |
|----------------------------|------------------------------------------------------------------------------|
| `get`                      | Reads a value, allows updating state without making any changes.             |
| `get_or_insert`            | Gets the value for a key, inserting a given value if the key is absent.      |
| `get_or_insert_with`       | Gets the value for a key, inserting a generated value if the key is absent.  |
| `insert_with`              | Insert a value generated from the key.                                       |
| `insert_with_if_absent`    | Insert a value generated from the key, only if the key is absent.            |
| `modify`                   | Mutate an existing value in-place. Does nothing if key absent.               |
| `move_value`               | Remove a value from one key and insert it with another key.                  |
| `remove`                   | Remove the given key.                                                        |
| `remove_if`                | Remove the given key if it also satisfies a condition.                       |
| `swap_value`               | Swap the values of two keys.                                                 |
| `update`                   | Update a single entry. Return `Some(v)` to insert/replace, `None` to remove. |

### TxMap operations

All transaction operations (or variations of them) are also available on TxMap.
There are also some additional operations that are only available on TxMap which are as follows

| TxMap operation | Description                                   |
|-----------------|-----------------------------------------------|
| `capacity`      | Returns the total capacity of all shards      |
| `clear`         | Removes all entries                           |
| `contains_key`  | Returns if the map contains the key           |
| `drain`         | Removes and returns all the entries           |
| `fold`          | Performs a fold on all the entries            |
| `hasher`        | Returns the hasher builder                    |
| `into_keys`     | Consumes the map, returning its keys          |
| `into_values`   | Consumes the map, returning its values        |
| `is_empty`      | Returns if the map is empty                   |
| `iter`          | Creates an iterator over all the entries      |
| `keys`          | Creates an iterator over all the keys         |
| `len`           | Returns how many entries the map contains     |
| `remove_entry`  | Removes a key and returns key and value       |
| `reserve`       | Reserves capacity for more entries            |
| `retain`        | Retains any entry which satisfies a condition |
| `shrink_to`     | Shrinks capacity to a lower bound             |
| `shrink_to_fit` | Shrinks capacity as much as possible          |
| `try_reserve`   | Tries to reserve capacity for more entries    |
| `values`        | Creates an iterator over all the values       |

`TxMap` also implements the standard collection traits: `Clone`, `PartialEq`, `Eq`, `Debug`, `Extend`, `FromIterator`, `From<[(K, V); N]>` and `IntoIterator`.

For a detailed comparison of the `TxMap` API surface with `std::collections::HashMap` see [docs/api_comparison.md](docs/api_comparison.md).
