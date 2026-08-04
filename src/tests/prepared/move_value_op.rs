use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn move_existing_value() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .move_value(GetTwo::a, GetTwo::b)
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
                result_a: None,
                result_b: Some(1)
            }
        }
    );
}

#[test]
fn move_value_overwrites_existing() {
    let map = map_alice_bob(1, 2);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .move_value(GetTwo::a, GetTwo::b)
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
                result_a: None,
                result_b: Some(1)
            }
        }
    );
}

#[test]
fn move_none_removes_existing() {
    let map = map_alice(1);
    let tx = map
        .prepared_tx(&GetTwo::SCHEMA)
        .move_value(GetTwo::a, GetTwo::b)
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
                a: BOB.into(),
                b: ALICE.into()
            },
            GetTwoParams {}
        ),
        TxResult::Completed {
            state: GetTwoState {
                result_a: None,
                result_b: None
            }
        }
    );
}

#[test]
fn move_value_to_self() {
    let map = empty_map();
    map.insert(ALICE.into(), 7);
    let tx = map
        .prepared_tx(&GetOne::SCHEMA)
        .move_value(GetOne::key, GetOne::key)
        .get(GetOne::key, |_k, v, _p, s| {
            s.result = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(GetOneKeys { key: ALICE.into() }, GetOneParams {}),
        TxResult::Completed {
            state: GetOneState { result: Some(7) }
        }
    );
}

#[test]
fn param_move_value() {
    let map = map_alice(42);
    let tx = map
        .prepared_tx(&GetTwoParam::SCHEMA)
        .move_value(GetTwoParam::a, GetTwoParam::b)
        .get(GetTwoParam::a, |_k, v, _p, s| {
            s.result_a = v.copied();
        })
        .get(GetTwoParam::b, |_k, v, _p, s| {
            s.result_b = v.copied();
        })
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetTwoParamKeys {
                a: ALICE.into(),
                b: BOB.into()
            },
            GetTwoParamParams { _p: () }
        ),
        TxResult::Completed {
            state: GetTwoParamState {
                result_a: None,
                result_b: Some(42)
            }
        }
    );
}
