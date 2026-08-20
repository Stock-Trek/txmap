use crate::tests::{creators::*, data::*};

#[test]
fn iter_empty_map() {
    let map = empty_map();
    let mut count = 0;
    for _ in map.iter() {
        count += 1;
    }
    assert_eq!(count, 0);
}

#[test]
fn iter_single_entry() {
    let map = map_alice(42);
    let mut entries: Vec<(String, u64)> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort();
    assert_eq!(entries, vec![(ALICE.into(), 42)]);
}

#[test]
fn iter_multiple_entries() {
    let map = map_alice_bob_chuck(10, 20, 30);
    let mut entries: Vec<(String, u64)> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![(ALICE.into(), 10), (BOB.into(), 20), (CHUCK.into(), 30),]
    );
}

#[test]
fn iter_matches_len() {
    let map = map_alice_bob_chuck_dave(1, 2, 3, 4);
    let count = map.iter().count();
    assert_eq!(count, map.len());
}

#[test]
fn iter_via_into_iterator() {
    let map = map_alice_bob(100, 200);
    let mut entries: Vec<(String, u64)> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort();
    assert_eq!(entries, vec![(ALICE.into(), 100), (BOB.into(), 200),]);
}

#[test]
fn iter_size_hint() {
    let map = map_alice_bob_chuck(1, 2, 3);
    let mut iter = map.iter();
    let mut yielded = 0;
    loop {
        // The hint must never over-promise: the lower bound cannot exceed the
        // true remaining count and the upper bound (when present) cannot be
        // below it. With lazily acquired shard locks the exact count is only
        // known once every shard has been locked.
        let (lower, upper) = iter.size_hint();
        assert!(lower <= 3 - yielded);
        if let Some(upper) = upper {
            assert!(upper >= 3 - yielded);
        }
        match iter.next() {
            Some(_) => yielded += 1,
            None => break,
        }
    }
    assert_eq!(yielded, 3);
    // Once fully consumed every shard is locked and the hint is exact.
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn for_loop_syntax() {
    let map = map_alice_bob(5, 10);
    let mut sum = 0u64;
    for (_k, v) in &map {
        sum += *v;
    }
    assert_eq!(sum, 15);
}
