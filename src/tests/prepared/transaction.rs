use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn empty_key_works() {
    let map = empty_map();
    map.insert("".into(), 1);
    assert_eq!(map.get_with(&"".into(), |v| *v), Some(1));
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v += 1)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: "".into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(2) }
        }
    );
}

#[test]
fn transaction_on_empty_map() {
    let map = empty_map();
    let result = map
        .prepared_tx(&GetOne::SCHEMA)
        .modify(GetOne::key, |_k, v, _p, _s| *v = 42)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction()
        .execute(GetOneKeys { key: ALICE.into() }, GetOneParams {});
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneState { result: None }
        }
    );
}

#[test]
fn mixed_ops_in_one_transaction() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetThree::SCHEMA)
        .insert_with(GetThree::a, |_k, _p, _s| 123)
        .insert_with(GetThree::b, |_k, _p, _s| 123)
        .insert_with(GetThree::c, |_k, _p, _s| 123)
        .modify(GetThree::a, |_k, v, _p, _s| *v = 10)
        .modify(GetThree::b, |_k, v, _p, _s| *v = 20)
        .update(GetThree::c, |_k, _v, _p, _s| Some(30))
        .get(GetThree::a, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .get(GetThree::b, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .get(GetThree::c, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetThreeKeys {
                a: ALICE.into(),
                b: BOB.into(),
                c: CHUCK.into()
            },
            GetThreeParams {}
        ),
        TxResult::Completed {
            state: GetThreeState {
                results: vec![Some(10), Some(20), Some(30)]
            }
        }
    );
}

#[test]
fn chain_many_ops() {
    let map: TxMap<u64, u64> = empty_typed_map();
    for i in 0..5u64 {
        let tx = map
            .prepared_tx(&Increment::SCHEMA)
            .insert_with(Increment::k, |_k, _p, _s| 123)
            .into_transaction();
        assert_eq!(
            tx.execute(IncrementKeys { k: i }, IncrementParams {}),
            TxResult::Completed {
                state: IncrementState {}
            }
        );
    }
    assert_eq!(map.len(), 5);
}

#[test]
fn chain_many_ops_with_params() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetVecParam::SCHEMA)
        .insert_with(GetVecParam::a, |_k, _p, _s| 123)
        .insert_with(GetVecParam::b, |_k, _p, _s| 123)
        .modify(GetVecParam::a, |_k, v, p, _s| *v = p.param[0])
        .modify(GetVecParam::b, |_k, v, p, _s| *v = p.param[1])
        .get(GetVecParam::a, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .get(GetVecParam::b, |_k, v, _p, s| {
            s.results.push(v.copied());
        })
        .into_transaction();
    let result = tx.execute(
        GetVecParamKeys {
            a: ALICE.into(),
            b: BOB.into(),
        },
        GetVecParamParams {
            param: vec![10, 20],
        },
    );
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetVecParamState {
                results: vec![Some(10), Some(20)]
            }
        }
    );
}

#[test]
fn chained_modify_and_get() {
    let map: TxMap<String, Counter> = empty_typed_map();
    let tx = map
        .prepared_tx(&GetCounter::SCHEMA)
        .insert_with(GetCounter::key, |_k, _p, _s| Counter::default())
        .modify(GetCounter::key, |_k, c, _p, _s| c.value += 1)
        .modify(GetCounter::key, |_k, c, _p, _s| c.value += 1)
        .get(GetCounter::key, |_k, c, _p, s| {
            s.result = c.as_ref().map(|c| c.value);
        })
        .into_transaction();
    let result = tx.execute(GetCounterKeys { key: "ctr".into() }, GetCounterParams {});
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetCounterState { result: Some(2) }
        }
    );
}

#[test]
fn chained_ops_on_multiple_keys() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .insert_with(GetTwo::a, |_k, _p, _s| 0)
        .insert_with(GetTwo::b, |_k, _p, _s| 0)
        .modify(GetTwo::a, |_k, v, _p, _s| *v += 10)
        .modify(GetTwo::b, |_k, v, _p, _s| *v += 20)
        .get(GetTwo::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .get(GetTwo::b, |_k, v, _p, s| {
            s.result_b = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetTwoKeys {
                a: ALICE.into(),
                b: BOB.into()
            },
            GetTwoParams {}
        ),
        TxResult::Completed {
            state: GetTwoState {
                result_a: Some(10),
                result_b: Some(20)
            }
        }
    );
}

#[test]
fn param_transaction_basic() {
    let map = map_alice(0);
    let tx = map
        .prepared_tx(&GetOneParamU64::SCHEMA)
        .modify(GetOneParamU64::key, |_k, v, p, _s| *v += p.param)
        .get(GetOneParamU64::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys { key: ALICE.into() },
            GetOneParamU64Params { param: 50 }
        ),
        TxResult::Completed {
            state: GetOneParamU64State { result: Some(50) }
        }
    );
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys { key: ALICE.into() },
            GetOneParamU64Params { param: 30 }
        ),
        TxResult::Completed {
            state: GetOneParamU64State { result: Some(80) }
        }
    );
}

/// A prepared transaction can be stored by naming only the map's key and
/// value types. The specialised `TransferPreparedTransaction` type is
/// generated by `tx_schema!` and is generic over the key and value types
/// (the lock policy and hasher default to `MutexPolicy` and
/// `DefaultBuildHasher`), so the schema itself is not a generic parameter.
#[test]
fn prepared_transaction_is_storable_with_key_and_value_types() {
    struct App<'tx> {
        transfer: TransferTx<'tx, String, u64>,
    }
    impl<'tx> App<'tx> {
        fn new(map: &'tx TxMap<String, u64>) -> App<'tx> {
            App {
                transfer: Transfer::prepared_tx(map)
                    .modify(Transfer::from, |_k, v, p, _s| *v -= p.amount)
                    .modify(Transfer::to, |_k, v, p, _s| *v += p.amount)
                    .into_transaction(),
            }
        }
    }

    let map = empty_map();
    map.insert(ALICE.into(), 100);
    map.insert(BOB.into(), 0);
    let app = App::new(&map);
    let result = app.transfer.execute(
        TransferKeys {
            from: ALICE.into(),
            to: BOB.into(),
        },
        TransferParams { amount: 40 },
    );
    assert!(matches!(result, TxResult::Completed { .. }));
    assert_eq!(map.get_copied(&ALICE.into()), Some(60));
    assert_eq!(map.get_copied(&BOB.into()), Some(40));
    // Reusable.
    let result = app.transfer.execute(
        TransferKeys {
            from: ALICE.into(),
            to: BOB.into(),
        },
        TransferParams { amount: 10 },
    );
    assert!(matches!(result, TxResult::Completed { .. }));
    assert_eq!(map.get_copied(&ALICE.into()), Some(50));
    assert_eq!(map.get_copied(&BOB.into()), Some(50));
}

/// The macro-generated `IncrementPreparedTransaction` type is storable by
/// naming only the map's key and value types. There is no standalone generic
/// `PreparedTransaction` type anymore; every schema gets its own
/// macro-generated type.
#[test]
fn prepared_transaction_is_storable_with_key_and_value_types_via_map_entry() {
    struct App<'tx> {
        increment: IncrementTx<'tx, String, u64>,
    }
    impl<'tx> App<'tx> {
        fn new(map: &'tx TxMap<String, u64>) -> App<'tx> {
            App {
                increment: map
                    .prepared_tx(&Increment::SCHEMA)
                    .modify(Increment::k, |_k, v, _p, _s| *v += 1)
                    .into_transaction(),
            }
        }
    }

    let map = empty_map();
    map.insert(ALICE.into(), 0);
    let app = App::new(&map);
    let _ = app
        .increment
        .execute(IncrementKeys { k: ALICE.into() }, IncrementParams {});
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

/// The specialised `prepared_tx` method generated by `tx_schema!` lives on
/// the schema type and needs no generic parameters at the call site: every
/// type is inferred from the map. The returned builder is the specialised
/// `TransferPreparedTxBuilder` type.
#[test]
fn specialised_prepared_tx_method_needs_no_generics() {
    let map = empty_map();
    map.insert(ALICE.into(), 100);
    map.insert(BOB.into(), 0);

    let builder: TransferBuilder<'_, Transfer<String, u64>> = Transfer::prepared_tx(&map);
    let tx = builder
        .modify(Transfer::from, |_k, v, p, _s| *v -= p.amount)
        .modify(Transfer::to, |_k, v, p, _s| *v += p.amount)
        .into_transaction();
    let result = tx.execute(
        TransferKeys {
            from: ALICE.into(),
            to: BOB.into(),
        },
        TransferParams { amount: 25 },
    );
    assert!(matches!(result, TxResult::Completed { .. }));
    assert_eq!(map.get_copied(&ALICE.into()), Some(75));
    assert_eq!(map.get_copied(&BOB.into()), Some(25));
}

/// Every macro-generated `PreparedTransaction` implements
/// `PreparedTransactionTrait`, exposing the schema as the `SCHEMA` associated
/// type. This lets generic code bound on the trait and still name the schema
/// (and its keys/params/state types) without the schema being a generic
/// parameter of the transaction type.
#[test]
fn prepared_transaction_implements_prepared_transaction_trait() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&Transfer::SCHEMA)
        .modify(Transfer::from, |_k, v, p, _s| *v -= p.amount)
        .modify(Transfer::to, |_k, v, p, _s| *v += p.amount)
        .into_transaction();

    // The schema is recoverable from the stored transaction type via the
    // trait's associated type, without naming it as a generic parameter.
    fn assert_schema<T: TxTrait>(_tx: &T) {
        // `T::SCHEMA` is a `TxSchema` and its structural types are usable.
        fn _check<KEYS, PARAMS, STATE: Default>() {}
        _check::<
            <T::SCHEMA as TxSchema>::Keys,
            <T::SCHEMA as TxSchema>::Params,
            <T::SCHEMA as TxSchema>::State,
        >();
    }
    assert_schema(&tx);

    map.insert(ALICE.into(), 100);
    map.insert(BOB.into(), 0);
    let result = tx.execute(
        TransferKeys {
            from: ALICE.into(),
            to: BOB.into(),
        },
        TransferParams { amount: 30 },
    );
    assert!(matches!(result, TxResult::Completed { .. }));
    assert_eq!(map.get_copied(&ALICE.into()), Some(70));
    assert_eq!(map.get_copied(&BOB.into()), Some(30));
}
