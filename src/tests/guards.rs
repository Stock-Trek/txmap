#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{
            creators::creators::{empty_map, empty_typed_map, map_alice, map_alice_bob},
            data::data::{ALICE, BOB},
        },
    };

    #[test]
    fn empty_values_in_guard() {
        let map = empty_map();
        let tx = map
            .transaction()
            .require("Alice exists", [ALICE.into()], |[v]| v.is_some())
            .modify("k".into(), |_, _| {})
            .into_transaction();
        assert!(matches!(tx.execute(), TxResult::RequirementNotMet(0, _)));
    }

    #[test]
    fn same_key() {
        let map = map_alice(5);
        let result = map
            .transaction()
            .require(
                "Both are Alice",
                [ALICE.into(), ALICE.into()],
                |[v1, v2]| v1 == v2 && v1.is_some_and(|x| *x == 5),
            )
            .modify(ALICE.into(), |_, _| {})
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(()));
    }

    #[test]
    fn one_failed_requirement_can_veto_transaction() {
        let map = map_alice_bob(1, 2);
        let result = map
            .transaction()
            .require("Alice > 0", [ALICE.into()], |[v]| v.is_some_and(|x| *x > 0))
            .require("Bob > 0", [BOB.into()], |[v]| v.is_some_and(|x| *x > 0))
            .require("Alice == 1", [ALICE.into()], |[v]| {
                v.is_some_and(|x| *x == 1)
            })
            .require("Bob == 1", [BOB.into()], |[v]| v.is_some_and(|x| *x == 1))
            .modify(ALICE.into(), |_, _| {})
            .into_transaction()
            .execute();
        assert!(matches!(result, TxResult::RequirementNotMet(3, _)));
    }

    #[test]
    fn keys_in_same_shard() {
        let map = empty_typed_map::<u64, u64>();
        for i in 0..100 {
            map.insert(i, i);
        }
        let keys = std::array::from_fn::<u64, 100, _>(|i| i as u64);
        let tx = map
            .transaction()
            .require("sum", keys, |values| {
                for i in 0..100 {
                    if values[i].is_none_or(|v| *v != i as u64) {
                        return false;
                    }
                }
                true
            })
            .modify(0, |_k, v| *v += 0)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }
}
