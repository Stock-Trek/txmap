#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{creators::creators::map_alice, data::data::ALICE},
    };

    #[test]
    fn param_transaction_basic() {
        let map = map_alice(0);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .modify(ALICE.into(), |_k, v, param| *v += param)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&50), TxResult::Completed(Some(50)));
        assert_eq!(tx.execute(&30), TxResult::Completed(Some(80)));
    }

    #[test]
    fn param_requirement_not_met() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("funds".into(), 100);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .require("sufficient", ["funds".into()], |[v], min| {
                v.copied().unwrap_or(0) >= *min
            })
            .modify("funds".into(), |_k, v, _p| *v += 0)
            .into_transaction();
        assert_eq!(tx.execute(&50), TxResult::Completed(()));
        assert!(matches!(
            tx.execute(&200),
            TxResult::RequirementNotMet(0, _)
        ));
    }

    #[test]
    fn param_insert_with() {
        let map: TxMap<String, String> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .with_param::<String>()
            .insert_with("k".into(), |_k, param| param.clone())
            .into_transaction();
        assert_eq!(tx.execute(&"hello".into()), TxResult::Completed(()));
        assert_eq!(
            map.get_with(&"k".into(), |v| v.clone()),
            Some("hello".into())
        );
    }

    #[test]
    fn param_map_op() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .update("k".into(), |_k, v, mult| v.map(|x| x * mult))
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&3), TxResult::Completed(Some(30)));
    }

    #[test]
    fn param_remove_where() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 5);
        map.insert("b".into(), 15);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .remove_where(["a".into(), "b".into()], |_k, v, threshold| *v > *threshold)
            .into_transaction();
        assert_eq!(tx.execute(&10), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(5));
    }

    #[test]
    fn param_retain_where() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 5);
        map.insert("b".into(), 15);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .retain_where(["a".into(), "b".into()], |_k, v, threshold| {
                *v >= *threshold
            })
            .into_transaction();
        assert_eq!(tx.execute(&10), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(15));
    }

    #[test]
    fn param_modify_peek() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        map.insert("p".into(), 5);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .modify_peek("k".into(), ["p".into()], |_k, v, [p], mult| {
                *v = p.copied().unwrap_or(0) * mult
            })
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&3), TxResult::Completed(Some(15)));
    }

    #[test]
    fn param_swap_value() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .swap_value("a".into(), "b".into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(2));
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(1));
    }

    #[test]
    fn param_move_value() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 42);
        let tx = map
            .transaction()
            .with_param::<()>()
            .move_value("a".into(), "b".into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(42));
        assert_eq!(map.get_with(&"a".into(), |v| *v), None);
    }

    #[test]
    fn param_get_all() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        let tx = map
            .transaction()
            .with_param::<()>()
            .modify("a".into(), |_k, v, _p| *v += 0)
            .modify("b".into(), |_k, v, _p| *v += 0)
            .get_all(["a".into(), "b".into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(vec![Some(10), None]));
    }

    #[test]
    fn param_insert_default() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .with_param::<()>()
            .insert_default("k".into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.get_with(&"k".into(), |v| *v), Some(0));
    }

    #[test]
    fn param_update_peek() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        map.insert("p".into(), 5);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .update_peek(
                "k".into(),
                |_k, v, [p], mult| v.map(|x| (x + p.unwrap_or(&0)) * mult),
                ["p".into()],
            )
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&2), TxResult::Completed(Some(30)));
    }
}
