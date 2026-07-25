use crate::{
    prelude::*,
    tests::{
        creators::*,
        data::*,
        types::{Increment, IncrementKeys, IncrementParams},
    },
};

#[test]
fn empty_values_in_guard() {
    let map = empty_map();
    let tx = map
        .prepare_transaction(&Increment::SCHEMA)
        .require("Alice exists", Increment::k, |v, _p, _s| v.is_some())
        .modify(Increment::k, |_k, _v, _p, _s| {})
        .into_transaction();
    assert!(matches!(
        tx.execute(IncrementKeys { k: ALICE.into() }, IncrementParams {}),
        TxResult::RequirementNotMet(0, _)
    ));
}

#[test]
fn one_failed_requirement_can_veto_transaction() {
    let map = map_alice(1);
    let result = map
        .prepare_transaction(&Increment::SCHEMA)
        .require("Exists", Increment::k, |v, _p, _s| v.is_some())
        .require("> 0", Increment::k, |v, _p, _s| v.is_some_and(|x| *x > 0))
        .require("== 1", Increment::k, |v, _p, _s| v.is_some_and(|x| *x == 1))
        .require("> 99", Increment::k, |v, _p, _s| v.is_some_and(|x| *x > 99))
        .modify(Increment::k, |_, v, _, _| *v = 100)
        .into_transaction()
        .execute(IncrementKeys { k: ALICE.into() }, IncrementParams {});
    assert!(matches!(result, TxResult::RequirementNotMet(3, _)));
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}
