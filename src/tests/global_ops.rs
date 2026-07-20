#[cfg(test)]
mod global_ops {
    use crate::prelude::*;

    #[test]
    fn clear_via_transaction() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map.transaction().clear().into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert!(map.is_empty());
    }

    #[test]
    fn remove_if_removes_matching() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .remove_if(|_k, v| *v > 1)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_keeps_matching() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .retain(|_k, v| *v % 2 == 0)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(2));
    }

    #[test]
    fn remove_if_empty_map() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .remove_if(|_k, _v| true)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }

    #[test]
    fn retain_all_on_empty_map() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map.transaction().retain(|_k, _v| false).into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }

    #[test]
    fn param_clear() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
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
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
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
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        map.insert("b".into(), 20);
        map.insert("c".into(), 30);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .retain(|_k, v, min| *v >= *min)
            .into_transaction();
        assert_eq!(tx.execute(&25), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }
}
