#[cfg(test)]
pub(crate) use crate::shard_count::ShardCount;
#[cfg(test)]
pub(crate) use crate::tests::data::{ALICE, BOB, CHUCK, DAVE};
#[cfg(test)]
pub(crate) use crate::tx_map::TxMap;
#[cfg(test)]
pub(crate) use rand::seq::SliceRandom;
#[cfg(test)]
pub(crate) use std::hash::Hash;

#[cfg(test)]
pub(crate) fn empty_typed_map<K, V>() -> TxMap<K, V>
where
    K: Hash + Eq,
{
    TxMap::new(ShardCount::_8)
}

#[cfg(test)]
pub(crate) fn empty_map() -> TxMap<String, u64> {
    TxMap::new(ShardCount::_8)
}

#[cfg(test)]
pub(crate) fn map_alice(alice: u64) -> TxMap<String, u64> {
    let map = empty_typed_map();
    map.insert(ALICE.into(), alice);
    map
}

#[cfg(test)]
pub(crate) fn map_alice_bob(alice: u64, bob: u64) -> TxMap<String, u64> {
    let map = empty_typed_map();
    map.insert(ALICE.into(), alice);
    map.insert(BOB.into(), bob);
    map
}

#[cfg(test)]
pub(crate) fn map_alice_bob_chuck(alice: u64, bob: u64, chuck: u64) -> TxMap<String, u64> {
    let map = empty_typed_map();
    map.insert(ALICE.into(), alice);
    map.insert(BOB.into(), bob);
    map.insert(CHUCK.into(), chuck);
    map
}

#[cfg(test)]
pub(crate) fn map_alice_bob_chuck_dave(
    alice: u64,
    bob: u64,
    chuck: u64,
    dave: u64,
) -> TxMap<String, u64> {
    let map = empty_typed_map();
    map.insert(ALICE.into(), alice);
    map.insert(BOB.into(), bob);
    map.insert(CHUCK.into(), chuck);
    map.insert(DAVE.into(), dave);
    map
}

#[cfg(test)]
pub(crate) fn random_names<const C: usize>() -> [String; C] {
    assert!(C <= 4);
    let rng = &mut rand::rng();
    let mut vec = vec![ALICE.into(), BOB.into(), CHUCK.into(), DAVE.into()];
    vec.shuffle(rng);
    vec.truncate(C);
    vec.try_into().unwrap()
}
