use crate::tests::{creators::*, data::*};

#[test]
fn retain_on_empty_map() {
    let map = empty_map();
    map.retain(|_k, _v| false);
    assert_eq!(map.len(), 0);
}

#[test]
fn remove_empty_map() {
    let map = empty_map();
    let previous_value = map.remove(&ALICE.into());
    assert_eq!(previous_value, None);
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
