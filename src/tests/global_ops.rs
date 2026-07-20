#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{
            creators::creators::empty_map,
            data::data::{ALICE, BOB, CHUCK},
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
}
