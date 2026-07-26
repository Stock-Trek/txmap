#[cfg(test)]
mod tests {
    use crate::tests::{creators::*, data::*};

    #[test]
    fn retain_on_empty_map() {
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
}
