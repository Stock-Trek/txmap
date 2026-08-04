use crate::{
    prelude::*,
    tests::{creators::*, types::*},
};

#[test]
fn require_condition_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 100);
    let threshold = 50u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .require("sufficient", "funds".into(), move |_k, v, _s| {
            v.copied().unwrap_or(0) >= threshold
        })
        .modify("funds".into(), |_k, v, s| {
            *v -= 30;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneParamU64State { result: Some(70) }
        }
    );
}

#[test]
fn require_condition_not_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 30);
    let threshold = 50u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .require("sufficient", "funds".into(), move |_k, v, _s| {
            v.copied().unwrap_or(0) >= threshold
        })
        .modify("funds".into(), |_k, v, _s| *v -= 30)
        .execute();
    assert!(matches!(
        result,
        TxResult::RequirementNotMet {
            index: 0,
            requirement: _,
            state: _
        }
    ));
}

#[test]
fn param_requirement_not_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 100);
    let threshold = 200u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .require("sufficient", "funds".into(), move |_k, v, _s| {
            v.copied().unwrap_or(0) >= threshold
        })
        .modify("funds".into(), |_k, v, _s| *v += 0)
        .execute();
    assert!(matches!(
        result,
        TxResult::RequirementNotMet {
            index: 0,
            requirement: _,
            state: _
        }
    ));
}
