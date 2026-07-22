#[cfg(test)]
mod tests {
    use crate::tests::{
        creators::creators::{empty_map, map_alice_bob_chuck},
        data::data::{ALICE, BOB, CHUCK},
    };

    #[test]
    fn retain_only_keeps_specified() {
        let map = map_alice_bob_chuck(1, 2, 3);
        map.retain_only([ALICE.into(), BOB.into()]);
        let chuck = map.get_copied(&CHUCK.into());
        assert_eq!(chuck, None);
        assert_eq!(map.len(), 2);
    }
    #[test]
    fn retain_all_on_empty_map() {
        let map = empty_map();
        map.retain(|_k, _v| false);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn remove_if_empty_map() {
        let map = empty_map();
        map.remove_if(|_k, _v| true);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn retain_keeps_matching() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.insert(BOB.into(), 2);
        map.insert(CHUCK.into(), 3);
        map.retain(|_k, v| *v % 2 == 0);
        assert_eq!(map.get_copied(&BOB.into()), Some(2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_if_removes_matching() {
        let map = empty_map();
        map.insert(ALICE.into(), 1);
        map.insert(BOB.into(), 2);
        map.insert(CHUCK.into(), 3);
        map.remove_if(|_k, v| *v > 1);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_where_keeps_matching() {
        let map = map_alice_bob_chuck(1, 2, 3);
        map.retain_where([ALICE.into(), BOB.into(), CHUCK.into()], |_k, v| *v >= 2);
        assert_eq!(
            map.get_all_copied([ALICE.into(), BOB.into(), CHUCK.into()]),
            vec![None, Some(2), Some(3)]
        );
        assert_eq!(map.len(), 2);
    }
}
