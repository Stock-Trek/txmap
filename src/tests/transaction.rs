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
    fn clear_via_transaction() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.insert(BOB.into(), 2);
        let tx = map.transaction().clear().into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert!(map.is_empty());
    }

    #[test]
    fn remove_if_removes_matching() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.insert(BOB.into(), 2);
        map.insert(CHUCK.into(), 3);
        let tx = map
            .transaction()
            .remove_if(|_k, v| *v > 1)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_keeps_matching() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.insert(BOB.into(), 2);
        map.insert(CHUCK.into(), 3);
        let tx = map
            .transaction()
            .retain(|_k, v| *v % 2 == 0)
            .get_copied(BOB.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(2)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_if_empty_map() {
        let map = empty_map();
        let tx = map
            .transaction()
            .remove_if(|_k, _v| true)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }

    #[test]
    fn retain_all_on_empty_map() {
        let map = empty_map();
        let tx = map.transaction().retain(|_k, _v| false).into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }

    #[test]
    fn param_clear() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        let tx = map
            .transaction()
            .with_param::<()>()
            .clear()
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert!(map.is_empty());
    }

    #[test]
    fn param_remove_if_global() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.insert(BOB.into(), 2);
        map.insert(CHUCK.into(), 3);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .remove_if(|_k, v, max| *v > *max)
            .into_transaction();
        assert_eq!(tx.execute(&1), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn param_retain_global() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        map.insert(BOB.into(), 20);
        map.insert(CHUCK.into(), 30);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .retain(|_k, v, min| *v >= *min)
            .get_copied(CHUCK.into())
            .into_transaction();
        assert_eq!(tx.execute(&25), TxResult::Completed(Some(30)));
        assert_eq!(map.len(), 1);
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
        let map = empty_map();
        let tx = map
            .transaction()
            .with_param::<Vec<u64>>()
            .insert_default(ALICE.into())
            .insert_default(BOB.into())
            .modify(ALICE.into(), |_k, v, p| *v = p[0])
            .modify(BOB.into(), |_k, v, p| *v = p[1])
            .get_all_copied([ALICE.into(), BOB.into()])
            .into_transaction();
        let result = tx.execute(&vec![10, 20]);
        assert_eq!(result, TxResult::Completed(vec![Some(10), Some(20)]));
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
}
