#[cfg(test)]
mod tests {
    use crate::tests::{
        creators::creators::{empty_map, map_alice, map_alice_bob, map_alice_bob_chuck},
        data::data::{ALICE, BOB},
    };

    #[test]
    fn new_map_is_empty() {
        let map = empty_map();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn insert_and_get_with() {
        let map = empty_map();
        assert_eq!(map.insert(ALICE.into(), 42), None);
        assert_eq!(map.insert(ALICE.into(), 99), Some(42));
        let val = map.get_with(&ALICE.into(), |v| *v);
        assert_eq!(val, Some(99));
    }

    #[test]
    fn remove_returns_value() {
        let map = empty_map();
        map.insert(ALICE.into(), 7);
        assert_eq!(map.remove(&ALICE.into()), Some(7));
        assert_eq!(map.remove(&ALICE.into()), None);
        assert!(map.is_empty());
    }

    #[test]
    fn clear_empties_map() {
        let map = map_alice_bob(1, 2);
        assert_eq!(map.len(), 2);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn fold_accumulates_matching_entries() {
        let map = map_alice_bob_chuck(3, 7, 29);
        let sum = map.fold(0u64, |_k, v| Some(*v), |acc, v| acc + v);
        assert_eq!(sum, 39);
        let filtered = map.fold(
            0u64,
            |k, v| if k.as_str() != BOB { Some(*v) } else { None },
            |acc, v| acc + v,
        );
        assert_eq!(filtered, 32);
    }

    #[test]
    fn get_with_on_missing_key() {
        let map = empty_map();
        let missing = map.get_with(&ALICE.into(), |v| *v);
        assert_eq!(missing, None);
    }

    #[test]
    fn insert_overwrites_and_returns_previous() {
        let map = map_alice(7);
        let previous_7 = map.insert(ALICE.into(), 42);
        assert_eq!(previous_7, Some(7));
        let previous_42 = map.insert(ALICE.into(), 100);
        assert_eq!(previous_42, Some(42));
    }
}
