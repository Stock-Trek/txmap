use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn empty_values_in_guard() {
    let map = empty_map();
    let tx = map
        .prepared_tx(&Increment::SCHEMA)
        .require("Alice exists", Increment::k, |_k, v, _p, _s| v.is_some())
        .modify(Increment::k, |_k, _v, _p, _s| {})
        .into_transaction();
    assert!(matches!(
        tx.execute(IncrementKeys { k: ALICE.into() }, IncrementParams {}),
        TxResult::RequirementNotMet(0, _, _)
    ));
}

#[test]
fn one_failed_requirement_can_veto_transaction() {
    let map = map_alice(1);
    let result = map
        .prepared_tx(&Increment::SCHEMA)
        .require("Exists", Increment::k, |_k, v, _p, _s| v.is_some())
        .require("> 0", Increment::k, |_k, v, _p, _s| {
            v.is_some_and(|x| *x > 0)
        })
        .require("== 1", Increment::k, |_k, v, _p, _s| {
            v.is_some_and(|x| *x == 1)
        })
        .require("> 99", Increment::k, |_k, v, _p, _s| {
            v.is_some_and(|x| *x > 99)
        })
        .modify(Increment::k, |_, v, _, _| *v = 100)
        .into_transaction()
        .execute(IncrementKeys { k: ALICE.into() }, IncrementParams {});
    assert!(matches!(result, TxResult::RequirementNotMet(3, _, _)));
    assert_eq!(map.get_copied(&ALICE.into()), Some(1));
}

#[test]
fn param_requirement_not_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 100);
    let tx = map
        .prepared_tx(&GetOneParamU64::SCHEMA)
        .require("sufficient", GetOneParamU64::key, |_k, v, p, _s| {
            v.copied().unwrap_or(0) >= p.param
        })
        .modify(GetOneParamU64::key, |_k, v, _p, _s| *v += 0)
        .into_transaction();
    assert_eq!(
        tx.execute(
            GetOneParamU64Keys {
                key: "funds".into()
            },
            GetOneParamU64Params { param: 50 }
        ),
        TxResult::Completed(GetOneParamU64State { result: None })
    );
    assert!(matches!(
        tx.execute(
            GetOneParamU64Keys {
                key: "funds".into()
            },
            GetOneParamU64Params { param: 200 }
        ),
        TxResult::RequirementNotMet(0, _, _)
    ));
}
