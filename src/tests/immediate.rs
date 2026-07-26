use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

// --- Transaction tests (immediate versions) ---

#[test]
fn immediate_empty_key_works() {
    let map = empty_map();
    map.insert("".into(), 1);
    assert_eq!(map.get_with(&"".into(), |v| *v), Some(1));
    let result = map
        .immediate_tx::<GetOneState>()
        .modify("".into(), |_k, v, s| {
            *v += 1;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: Some(2) }));
}

#[test]
fn immediate_transaction_on_empty_map() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v = 42;
            s.result = Some(*v);
        })
        .execute();
    // modify on missing key is noop, so result is None
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
}

#[test]
fn immediate_mixed_ops_in_one_transaction() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetThreeState>()
        .insert_default(ALICE.into())
        .insert_default(BOB.into())
        .insert_default(CHUCK.into())
        .modify(ALICE.into(), |_k, v, _s| *v = 10)
        .modify(BOB.into(), |_k, v, _s| *v = 20)
        .update(CHUCK.into(), |_k, _v, _s| Some(30))
        .get(ALICE.into(), |_k, v, s| s.results.push(v.copied()))
        .get(BOB.into(), |_k, v, s| s.results.push(v.copied()))
        .get(CHUCK.into(), |_k, v, s| s.results.push(v.copied()))
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetThreeState {
            results: vec![Some(10), Some(20), Some(30)]
        })
    );
}

#[test]
fn immediate_chain_many_ops() {
    let map: TxMap<u64, u64> = empty_typed_map();
    for i in 0..5u64 {
        let result = map
            .immediate_tx::<IncrementState>()
            .insert_default(i)
            .execute();
        assert_eq!(result, TxResult::Completed(IncrementState {}));
    }
    assert_eq!(map.len(), 5);
}

#[test]
fn immediate_chain_many_ops_with_params() {
    let map = empty_map();
    let p = vec![10u64, 20u64];
    let p2 = p.clone();
    let result = map
        .immediate_tx::<GetVecParamState>()
        .insert_default(ALICE.into())
        .insert_default(BOB.into())
        .modify(ALICE.into(), move |_k, v, s| {
            *v = p[0];
            s.results.push(Some(*v));
        })
        .modify(BOB.into(), move |_k, _v, s| {
            s.results.push(Some(p2[1]));
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetVecParamState {
            results: vec![Some(10), Some(20)]
        })
    );
}

#[test]
fn immediate_chained_modify_and_get() {
    let map: TxMap<String, Counter> = empty_typed_map();
    let result = map
        .immediate_tx::<GetCounterState>()
        .insert_default("ctr".into())
        .modify("ctr".into(), |_k, c, _s| c.value += 1)
        .modify("ctr".into(), |_k, c, _s| c.value += 1)
        .get("ctr".into(), |_k, c, s| {
            s.result = c.as_ref().map(|c| c.value);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetCounterState { result: Some(2) })
    );
}

#[test]
fn immediate_chained_ops_on_multiple_keys() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetTwoState>()
        .insert_default(ALICE.into())
        .insert_default(BOB.into())
        .modify(ALICE.into(), |_k, v, _s| *v += 10)
        .modify(BOB.into(), |_k, v, _s| *v += 20)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(10),
            result_b: Some(20)
        })
    );
}

// --- Parameterized tests (immediate versions) ---

#[test]
fn immediate_param_transaction_basic() {
    let map = map_alice(0);
    let param = 50u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += param;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneParamU64State { result: Some(50) })
    );
    // Second execution on same map with different captured param
    let param2 = 30u64;
    let result2 = map
        .immediate_tx::<GetOneParamU64State>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += param2;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result2,
        TxResult::Completed(GetOneParamU64State { result: Some(80) })
    );
}

#[test]
fn immediate_param_requirement_not_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 100);
    let threshold = 200u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .require("sufficient", "funds".into(), move |v, _s| {
            v.copied().unwrap_or(0) >= threshold
        })
        .modify("funds".into(), |_k, v, _s| *v += 0)
        .execute();
    assert!(matches!(result, TxResult::RequirementNotMet(0, _)));
}

#[test]
fn immediate_param_insert_with() {
    let map: TxMap<String, String> = empty_typed_map();
    let param = "hello".to_string();
    let result = map
        .immediate_tx::<GetOneParamStringState>()
        .insert_with(ALICE.into(), move |_k, _s| param.clone())
        .get(ALICE.into(), |_k, v, s| s.result = v.cloned())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneParamStringState {
            result: Some("hello".into())
        })
    );
}

#[test]
fn immediate_param_map_op() {
    let map = map_alice(10);
    let factor = 3u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .update(ALICE.into(), move |_k, v, s| {
            let r = v.map(|x| x * factor);
            s.result = r;
            r
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneParamU64State { result: Some(30) })
    );
}

#[test]
fn immediate_param_remove_where() {
    let map = map_alice_bob(5, 15);
    let threshold = 10u64;
    let result = map
        .immediate_tx::<GetTwoParamU64State>()
        .remove_where(ALICE.into(), move |_k, v, _s| *v > threshold)
        .remove_where(BOB.into(), move |_k, v, _s| *v > threshold)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(5),
            result_b: None
        })
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn immediate_param_modify_peek() {
    let map = map_alice_bob(10, 5);
    let factor = 3u64;
    let result = map
        .immediate_tx::<GetTwoParamU64State>()
        .modify(ALICE.into(), move |_k, v, s| {
            *v = 5 * factor;
            s.result_a = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(15),
            result_b: None
        })
    );
}

#[test]
fn immediate_param_swap_value() {
    let map = map_alice_bob(1, 2);
    let result = map
        .immediate_tx::<GetTwoParamState>()
        .swap_value(ALICE.into(), BOB.into())
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamState {
            result_a: Some(2),
            result_b: Some(1)
        })
    );
}

#[test]
fn immediate_param_move_value() {
    let map = map_alice(42);
    let result = map
        .immediate_tx::<GetTwoParamState>()
        .move_value(ALICE.into(), BOB.into())
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamState {
            result_a: None,
            result_b: Some(42)
        })
    );
}

#[test]
fn immediate_param_get_all() {
    let map = map_alice(10);
    let result = map
        .immediate_tx::<GetTwoParamState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 0)
        .modify(BOB.into(), |_k, v, _s| *v += 0)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamState {
            result_a: Some(10),
            result_b: None
        })
    );
}

#[test]
fn immediate_param_insert_default() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn immediate_param_update_peek() {
    let map = map_alice_bob(10, 5);
    let factor = 2u64;
    let result = map
        .immediate_tx::<GetTwoParamU64State>()
        .update(ALICE.into(), move |_k, v, s| {
            let r = v.map(|x| (x + 5) * factor);
            s.result_a = r;
            r
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoParamU64State {
            result_a: Some(30),
            result_b: None
        })
    );
}

// --- Insert tests (immediate versions) ---

#[test]
fn immediate_insert_with_creates_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with(ALICE.into(), |_k, _s| 42)
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn immediate_insert_with_overwrites_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with(ALICE.into(), |_k, _s| 42)
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn immediate_insert_with_if_absent_creates_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with_if_absent(ALICE.into(), |_k, _s| 42)
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(42) })
    );
}

#[test]
fn immediate_insert_with_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_with_if_absent(ALICE.into(), |_k, _s| 42)
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(1) })
    );
}

#[test]
fn immediate_insert_default_creates_default_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn immediate_insert_default_overwrites_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn immediate_insert_default_if_absent_creates_default_entry() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default_if_absent(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(0) })
    );
}

#[test]
fn immediate_insert_default_if_absent_does_not_overwrite_existing() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .insert_default_if_absent(ALICE.into())
        .get(ALICE.into(), |_k, v, s| s.result = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(1) })
    );
}

// --- Modify tests (immediate versions) ---

#[test]
fn immediate_modify_existing_key() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v += 5;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(6) })
    );
}

#[test]
fn immediate_modify_missing_key_is_noop() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v = 42;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: None })
    );
}

#[test]
fn immediate_modify_peek_existing_key() {
    let map = map_alice(1);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 5)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(6),
            result_b: None
        })
    );
}

#[test]
fn immediate_modify_peek_missing_key_is_noop() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify(ALICE.into(), |_k, v, _s| *v = 42)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: None,
            result_b: None
        })
    );
}

#[test]
fn immediate_modify_peek_can_use_peeked_values() {
    let map = map_alice_bob(1, 2);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 2)
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(3),
            result_b: None
        })
    );
}

#[test]
fn immediate_modify_peek_with_empty_peek_keys() {
    let map = empty_map();
    map.insert(ALICE.into(), 10);
    let result = map
        .immediate_tx::<GetOneState>()
        .modify(ALICE.into(), |_k, v, s| {
            *v = 99;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(99) })
    );
}

#[test]
fn immediate_modify_peek_modifies_with_peek_values() {
    let map = empty_map();
    map.insert("target".into(), 100);
    map.insert("reference".into(), 50);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify("target".into(), |_k, v, _s| *v += 50)
        .get("target".into(), |_k, v, s| s.result_a = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(150),
            result_b: None
        })
    );
}

#[test]
fn immediate_modify_peek_missing_target_is_noop() {
    let map = empty_map();
    map.insert("ref".into(), 99);
    let result = map
        .immediate_tx::<GetTwoState>()
        .modify("missing".into(), |_k, v, _s| *v = 0)
        .get("missing".into(), |_k, v, s| s.result_a = v.copied())
        .get("ref".into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: None,
            result_b: Some(99)
        })
    );
}

#[test]
fn immediate_modify_peek_modifies_using_peeked_values() {
    let map = empty_map();
    map.insert(ALICE.into(), 100);
    map.insert(BOB.into(), 20);
    map.insert(CHUCK.into(), 3);
    let result = map
        .immediate_tx::<GetThreeState>()
        .modify(ALICE.into(), |_k, v, _s| *v += 20 + 3)
        .get(ALICE.into(), |_k, v, s| s.results.push(v.copied()))
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetThreeState {
            results: vec![Some(123)]
        })
    );
}

#[test]
fn immediate_require_condition_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 100);
    let threshold = 50u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .require("sufficient", "funds".into(), move |v, _s| {
            v.copied().unwrap_or(0) >= threshold
        })
        .modify("funds".into(), |_k, v, s| {
            *v -= 30;
            s.result = Some(*v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneParamU64State { result: Some(70) })
    );
}

#[test]
fn immediate_require_condition_not_met() {
    let map = empty_typed_map::<String, u64>();
    map.insert("funds".into(), 30);
    let threshold = 50u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .require("sufficient", "funds".into(), move |v, _s| {
            v.copied().unwrap_or(0) >= threshold
        })
        .modify("funds".into(), |_k, v, _s| *v -= 30)
        .execute();
    assert!(matches!(result, TxResult::RequirementNotMet(0, _)));
}

#[test]
fn immediate_remove() {
    let map = map_alice(42);
    let result = map
        .immediate_tx::<GetOneState>()
        .remove(ALICE.into(), |entry, s| {
            s.result = entry.map(|(_, v)| v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: Some(42) })
    );
    assert!(map.is_empty());
}

#[test]
fn immediate_remove_missing_key() {
    let map = empty_map();
    let result = map
        .immediate_tx::<GetOneState>()
        .remove(ALICE.into(), |entry, s| {
            s.result = entry.map(|(_, v)| v);
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetOneState { result: None })
    );
}

#[test]
fn immediate_remove_where_condition_not_met() {
    let map = map_alice(5);
    let threshold = 10u64;
    let result = map
        .immediate_tx::<GetOneState>()
        .remove_where(ALICE.into(), move |_k, v, _s| *v > threshold)
        .execute();
    assert_eq!(result, TxResult::Completed(GetOneState { result: None }));
    assert_eq!(map.len(), 1);
}

#[test]
fn immediate_chain_get_only() {
    let map = map_alice_bob(10, 20);
    let result = map
        .immediate_tx::<GetTwoState>()
        .get(ALICE.into(), |_k, v, s| s.result_a = v.copied())
        .get(BOB.into(), |_k, v, s| s.result_b = v.copied())
        .execute();
    assert_eq!(
        result,
        TxResult::Completed(GetTwoState {
            result_a: Some(10),
            result_b: Some(20)
        })
    );
}
