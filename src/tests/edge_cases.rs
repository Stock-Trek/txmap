#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{
            creators::creators::{empty_map, empty_typed_map},
            data::data::{ALICE, BOB, CHUCK},
            types::types::Counter,
        },
    };

    #[test]
    fn large_number_of_keys() {
        let map: TxMap<u64, u64> = TxMap::new(ShardCount::_128);
        for i in 0..10_000 {
            map.insert(i, i * 3);
        }
        assert_eq!(map.len(), 10_000);
        // Verify a few values
        for i in (0..10_000).step_by(1000) {
            assert_eq!(map.get_with(&i, |v| *v), Some(i * 3));
        }
    }

    #[test]
    fn duplicate_keys_dont_cause_issues() {
        let map: TxMap<String, String> = empty_typed_map();
        let key: String = "same".into();
        for i in 0..100 {
            map.insert(key.clone(), format!("v{i}"));
        }
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&key, |v| v.clone()), Some("v99".into()));
    }

    #[test]
    fn empty_key_works() {
        let map = empty_map();
        map.insert("".into(), 1);
        assert_eq!(map.get_with(&"".into(), |v| *v), Some(1));
        let tx = map
            .transaction()
            .modify("".into(), |_k, v| *v += 1)
            .get_copied("".into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(2)));
    }

    #[test]
    fn transaction_on_empty_map() {
        let map = empty_map();
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v = 42)
            .get_copied(ALICE.into())
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(None));
    }

    #[test]
    fn mixed_ops_in_one_transaction() {
        let map = empty_map();
        let tx = map
            .transaction()
            .insert_default(ALICE.into())
            .insert_default(BOB.into())
            .insert_default(CHUCK.into())
            .modify(ALICE.into(), |_k, v| *v = 10)
            .modify(BOB.into(), |_k, v| *v = 20)
            .update(CHUCK.into(), |_k, _v| Some(30))
            .get_all_copied([ALICE.into(), BOB.into(), CHUCK.into()])
            .into_transaction();
        assert_eq!(
            tx.execute(),
            TxResult::Completed(vec![Some(10), Some(20), Some(30)])
        );
    }

    #[test]
    fn chain_many_ops() {
        let map: TxMap<u64, u64> = empty_typed_map();
        // Build transaction with multiple ops chained manually
        let tx = map
            .transaction()
            .insert_default(0)
            .insert_default(1)
            .insert_default(2)
            .insert_default(3)
            .insert_default(4)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn chain_many_ops_with_params() {
        let map: TxMap<u64, u64> = empty_typed_map();
        // Use a single transaction that modifies via with_param
        let tx = map
            .transaction()
            .with_param::<Vec<u64>>()
            .insert_default(0)
            .insert_default(1)
            .modify(0, |_k, v, p| *v = p[0])
            .modify(1, |_k, v, p| *v = p[1])
            .get_all_copied([0, 1])
            .into_transaction();
        let result = tx.execute(&vec![10, 20]);
        assert_eq!(result, TxResult::Completed(vec![Some(10), Some(20)]));
    }

    #[test]
    fn clear_then_reinsert() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.clear();
        assert!(map.is_empty());
        map.insert(ALICE.into(), 2);
        assert_eq!(map.get_with(&ALICE.into(), |v| *v), Some(2));
    }

    #[test]
    fn huge_string_keys() {
        let map = empty_map();
        let big_key = "x".repeat(10_000);
        map.insert(big_key.clone(), 42);
        assert_eq!(map.get_with(&big_key, |v| *v), Some(42));
    }

    #[test]
    fn swap_value_same_key() {
        let map = empty_map();
        map.insert(ALICE.into(), 7);
        let tx = map
            .transaction()
            .swap_value(ALICE.into(), ALICE.into())
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(7)));
    }

    #[test]
    fn move_value_to_self() {
        let map = empty_map();
        map.insert(ALICE.into(), 7);
        let tx = map
            .transaction()
            .move_value(ALICE.into(), ALICE.into())
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(7)));
    }

    #[test]
    fn modify_peek_with_empty_peek_keys() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [], |_k, v, []: [Option<&u64>; 0]| *v = 99)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(99)));
    }

    #[test]
    fn chained_modify_and_get() {
        let map: TxMap<String, Counter> = empty_typed_map();
        let tx = map
            .transaction()
            .insert_default("ctr".into())
            .modify("ctr".into(), |_k, c| c.value += 1)
            .modify("ctr".into(), |_k, c| c.value += 1)
            .get("ctr".into(), |_k, c| c.value)
            .into_transaction();
        let result = tx.execute();
        assert_eq!(result, TxResult::Completed(Some(2)));
    }

    #[test]
    fn chained_ops_on_multiple_keys() {
        let map = empty_map();
        let tx = map
            .transaction()
            .insert_default(ALICE.into())
            .insert_default(BOB.into())
            .modify(ALICE.into(), |_k, v| *v += 10)
            .modify(BOB.into(), |_k, v| *v += 20)
            .get_all_copied([ALICE.into(), BOB.into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![Some(10), Some(20)]));
    }

    #[test]
    fn get_returns_transformed_value() {
        let map = empty_map();
        map.insert(ALICE.into(), 7);
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v += 3)
            .get(ALICE.into(), |_k, v| *v * 2)
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(Some(20)));
    }

    #[test]
    fn get_returns_option_via_map_finisher() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v *= 2)
            .get_copied(ALICE.into())
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(Some(20)));
    }

    #[test]
    fn get_all_returns_multiple_values() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        map.insert(BOB.into(), 20);
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v += 0)
            .modify(BOB.into(), |_k, v| *v += 0)
            .modify(CHUCK.into(), |_k, v| *v += 0)
            .get_all_copied([ALICE.into(), BOB.into(), CHUCK.into()])
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(vec![Some(10), Some(20), None]));
    }

    #[test]
    fn modify_peek_modifies_with_peek_values() {
        let map = empty_map();
        map.insert("target".into(), 100);
        map.insert("reference".into(), 50);
        let tx = map
            .transaction()
            .modify_peek("target".into(), ["reference".into()], |_k, v, [ref_val]| {
                if let Some(r) = ref_val {
                    *v += r;
                }
            })
            .get_copied("target".into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(150)));
    }

    #[test]
    fn modify_peek_missing_target_is_noop() {
        let map = empty_map();
        map.insert("ref".into(), 99);
        let tx = map
            .transaction()
            .modify_peek("missing".into(), ["ref".into()], |_k, v, [_r]| *v = 0)
            .get_all_copied(["missing".into(), "ref".into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![None, Some(99)]));
    }

    #[test]
    fn modify_peek_modifies_using_peeked_values() {
        let map = empty_map();
        map.insert(ALICE.into(), 100);
        map.insert(BOB.into(), 20);
        map.insert(CHUCK.into(), 3);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into(), CHUCK.into()], |_k, v, [b, c]| {
                *v += *b.unwrap() + *c.unwrap();
            })
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(123)));
    }

    #[test]
    fn update_peek_modifies_based_on_peek() {
        let map = empty_map();
        map.insert("k".into(), 10);
        map.insert("p".into(), 5);
        let tx = map
            .transaction()
            .update_peek(
                "k".into(),
                |_k, v, [p]| v.map(|x| x + p.unwrap_or(&0)),
                ["p".into()],
            )
            .get_copied("k".into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(15)));
    }
}
