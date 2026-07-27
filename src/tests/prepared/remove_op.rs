use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn remove_multiple_keys() {
    let map = map_alice_bob_chuck(1, 2, 3);
    let tx = map
        .prepared_tx(&RemoveMultiple::SCHEMA)
        .remove(RemoveMultiple::a, |entry, _params, state| {
            state.user.push(entry.map(|e| e.0))
        })
        .remove(RemoveMultiple::b, |entry, _params, state| {
            state.user.push(entry.map(|e| e.0))
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            RemoveMultipleKeys {
                a: ALICE.into(),
                b: BOB.into(),
            },
            RemoveMultipleParams {},
        ),
        TxResult::Completed(RemoveMultipleState {
            user: vec![Some(ALICE.into()), Some(BOB.into())],
        })
    );
    assert_eq!(map.len(), 1);
}
