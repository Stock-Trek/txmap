use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

/// A key handle used by exactly one consuming operation (`insert_with`) is
/// provably last-used: the key is moved out of the per-execution keys
/// container instead of cloned. Verify repeated executions with different
/// keys behave identically.
#[test]
fn insert_with_single_use_handle_moves_key() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
        .into_transaction();
    for name in [ALICE, BOB, CHUCK] {
        assert_eq!(
            tx.execute(GetOneKeys { key: name.into() }, GetOneParams {}),
            TxResult::Completed {
                state: GetOneState { result: None }
            }
        );
        assert_eq!(map.get_copied(&name.into()), Some(42));
    }
}

/// Same as above for `insert_with_if_absent` on an empty and a populated map.
#[test]
fn insert_with_if_absent_single_use_handle_moves_key() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with_if_absent(GetOne::key, |_k, _p, _s| 42)
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: None }
        }
    );
    assert_eq!(map.get_copied(&ALICE.into()), Some(42));
    // Second execution on the same map: key already present, nothing changes.
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: None }
        }
    );
    assert_eq!(map.get_copied(&ALICE.into()), Some(42));
}

/// Same as above for `update` on an existing entry.
#[test]
fn update_single_use_handle_moves_key() {
    let map = map_alice_bob(1, 2);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .update(GetOne::key, |_k, v, _p, _s| v.map(|x| x * 10))
        .into_transaction();
    for name in [ALICE, BOB] {
        assert_eq!(
            tx.execute(GetOneKeys { key: name.into() }, GetOneParams {}),
            TxResult::Completed {
                state: GetOneState { result: None }
            }
        );
    }
    assert_eq!(map.get_copied(&ALICE.into()), Some(10));
    assert_eq!(map.get_copied(&BOB.into()), Some(20));
}

/// When a handle is used by several operations, the consuming operation must
/// fall back to cloning the key (taking it would break later uses).
#[test]
fn shared_handle_falls_back_to_cloning() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with_if_absent(GetOne::key, |_k, _p, _s| 42)
        .modify(GetOne::key, |_k, v, _p, _s| {
            *v += 1;
        })
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(43) }
        }
    );
    // Repeated execution: the fallback path must also be stable.
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(44) }
        }
    );
}

/// A handle used by several operations can still have its key moved by the
/// final consuming operation: iterating backwards, the last op to see a key
/// always takes it, while earlier uses clone.
#[test]
fn shared_handle_last_consuming_op_takes_key() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .get(GetOne::key, |_k, _v, _p, _s| {})
        .insert_with(GetOne::key, |_k, _p, _s| 42)
        .into_transaction();
    for name in [ALICE, BOB, CHUCK] {
        assert_eq!(
            tx.execute(GetOneKeys { key: name.into() }, GetOneParams {}),
            TxResult::Completed {
                state: GetOneState { result: None }
            }
        );
        assert_eq!(map.get_copied(&name.into()), Some(42));
    }
}

/// When two consuming operations share a handle, the later one takes the key
/// and the earlier one falls back to cloning.
#[test]
fn shared_handle_two_consuming_ops_last_takes_key() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
        .update(GetOne::key, |_k, v, _p, _s| v.map(|x| x + 1))
        .into_transaction();
    for name in [ALICE, BOB] {
        assert_eq!(
            tx.execute(GetOneKeys { key: name.into() }, GetOneParams {}),
            TxResult::Completed {
                state: GetOneState { result: None }
            }
        );
        assert_eq!(map.get_copied(&name.into()), Some(43));
    }
}

/// Guards are checked before any op runs, so a consuming op may still take a
/// key that a guard also references when it is the final user.
#[test]
fn guard_shared_handle_last_op_takes_key() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .require("always", GetOne::key, |_k, _v, _p, _s| true)
        .insert_with(GetOne::key, |_k, _p, _s| 42)
        .into_transaction();
    for name in [ALICE, BOB, CHUCK] {
        assert_eq!(
            tx.execute(GetOneKeys { key: name.into() }, GetOneParams {}),
            TxResult::Completed {
                state: GetOneState { result: None }
            }
        );
        assert_eq!(map.get_copied(&name.into()), Some(42));
    }
}
