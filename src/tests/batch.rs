#[cfg(test)]
mod batch {
    use crate::prelude::*;

    #[test]
    fn remove_multiple_keys() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .remove(["a".into(), "c".into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(2));
    }

    #[test]
    fn remove_where_conditionally() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .remove_where(["a".into(), "b".into(), "c".into()], |_k, v| *v >= 2)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(1));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_only_keeps_specified() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .retain_only(["a".into(), "b".into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_with(&"c".into(), |v| *v), None);
    }

    #[test]
    fn retain_where_keeps_matching() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        map.insert("b".into(), 20);
        map.insert("c".into(), 30);
        let tx = map
            .transaction()
            .retain_where(["a".into(), "b".into(), "c".into()], |_k, v| *v >= 20)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_with(&"a".into(), |v| *v), None);
    }

    #[test]
    fn param_remove() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .remove(["a".into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn param_retain_only() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .retain_only(["a".into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(1));
    }
}
