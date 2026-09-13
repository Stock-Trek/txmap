//! Split and merge protocols for the routing trie.
//!
//! These are the topology-changing operations described by the trie design:
//! a split turns one hot leaf into eight children using the next three hash
//! bits, and a merge collapses eight quiet sibling leaves back into one.
//!
//! Both operations follow the same ordering discipline as transactions:
//! acquire every affected leaf in a single all-or-nothing mask operation,
//! then take the per-leaf write locks. Because the mask acquisition is
//! all-or-nothing, two concurrent topology changes (and any overlapping
//! transaction) cannot deadlock.

use crate::{
    lock_policies::lock_policy::LockPolicy, new_types::HashCode, shard::Shard, trie::RoutingTrie,
    tx_map::TxMap,
};
use std::hash::{BuildHasher, Hash};

// These methods are only invoked by the crate's test suite until adaptive
// sharding is wired into the public API, so silence dead-code lints for
// non-test builds.
#[allow(dead_code)]
impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
{
    /// Splits the leaf that `hash_code` routes to.
    ///
    /// Returns `true` if the leaf was split, `false` if it could not be
    /// (the budget is exhausted or the routing changed concurrently).
    pub(crate) fn split_leaf(&self, hash_code: HashCode) -> bool {
        let map = &self.custodian.map;
        let source = map.route(hash_code.0);
        let Some(level) = map.routing.level_of(source) else {
            return false;
        };
        // Reserve the seven new children. This sets their active bits, but
        // not their ready bits, so snapshotting code cannot observe the
        // uninitialised slots.
        let Some(ids) = map.allocate_ids(7) else {
            return false;
        };
        let mut children = [0u8; 8];
        children[0] = source;
        children[1..].copy_from_slice(&ids[..7]);
        for &id in &ids[..7] {
            // SAFETY: `id` was just reserved and is not reachable from the
            // trie yet.
            unsafe { map.init_leaf(id, L::new(Shard::new())) };
        }

        let needed = children
            .iter()
            .fold(0u128, |mask, &id| mask | (1u128 << id));
        let seen = map.acquire_guard(needed);

        // Re-check that routing did not change while we waited for the mask.
        if map.routing.level_of(source) != Some(level) {
            drop(seen);
            for &id in &ids[..7] {
                // SAFETY: we reserved these slots and they are not routed to.
                unsafe { drop(map.take_leaf(id)) };
            }
            return false;
        }

        let mut guards: [Option<L::WriteGuard<'_, Shard<K, V>>>; 8] = std::array::from_fn(|_| None);
        // Take the per-leaf write locks in ascending id order. All other
        // multi-leaf code paths also lock in ascending order, so this can
        // never deadlock.
        let mut order: [usize; 8] = std::array::from_fn(|i| i);
        order.sort_by_key(|&i| children[i]);
        for &child in &order {
            // SAFETY: `seen` holds leaf `children[child]`.
            let lock = unsafe { map.leaf_ref(children[child]) };
            guards[child] = Some(L::write(lock));
        }

        // Redistribute the source leaf's entries over the eight children
        // using the three hash bits that the new branch consumes.
        let mut buckets: [Vec<(K, V)>; 8] = std::array::from_fn(|_| Vec::new());
        for (key, value) in guards[0].as_mut().expect("guarded").drain() {
            let child = RoutingTrie::child_index(self.indexer.hash(&key).0, level);
            buckets[child].push((key, value));
        }
        for (child, bucket) in buckets.iter_mut().enumerate().skip(1) {
            for (key, value) in bucket.drain(..) {
                let hash = self.indexer.hash(&key).0;
                guards[child].as_mut().expect("guarded").insert_unique(
                    hash,
                    (key, value),
                    |entry| self.indexer.hash(&entry.0).0,
                );
            }
        }
        for (key, value) in buckets[0].drain(..) {
            let hash = self.indexer.hash(&key).0;
            guards[0]
                .as_mut()
                .expect("guarded")
                .insert_unique(hash, (key, value), |entry| self.indexer.hash(&entry.0).0);
        }

        // Publish the new routing, then invalidate any transaction that
        // routed to the old leaf.
        let split = map.routing.split_leaf(source, children);
        debug_assert!(split.is_some(), "split of a locked leaf must succeed");
        map.bump_version(source);
        drop(guards);
        drop(seen);
        split.is_some()
    }

    /// Merges the eight sibling leaves that `hash_code` routes into.
    ///
    /// Returns `true` if the leaves were merged, `false` if the routing
    /// changed concurrently or no mergeable branch exists.
    pub(crate) fn merge_leaves(&self, hash_code: HashCode) -> bool {
        let map = &self.custodian.map;
        let Some((_, children)) = map.routing.siblings(hash_code.0) else {
            return false;
        };
        let needed = children
            .iter()
            .fold(0u128, |mask, &id| mask | (1u128 << id));
        let _seen = map.acquire_guard(needed);

        // Re-check that the branch still has exactly these children.
        if map.routing.siblings(hash_code.0).map(|(_, ids)| ids) != Some(children) {
            return false;
        }

        let mut guards: [Option<L::WriteGuard<'_, Shard<K, V>>>; 8] = std::array::from_fn(|_| None);
        // Ascending id order, matching every other multi-leaf lock path.
        let mut order: [usize; 8] = std::array::from_fn(|i| i);
        order.sort_by_key(|&i| children[i]);
        for &child in &order {
            // SAFETY: `seen` holds leaf `children[child]`.
            let lock = unsafe { map.leaf_ref(children[child]) };
            guards[child] = Some(L::write(lock));
        }

        // Everything funnels into the surviving child (child position 0).
        let survivor = children[0];
        let (survivor_slot, rest) = guards.split_at_mut(1);
        let survivor_guard = survivor_slot[0].as_mut().expect("guarded");
        for guard in rest {
            for (key, value) in guard.as_mut().expect("guarded").drain() {
                let hash = self.indexer.hash(&key).0;
                survivor_guard
                    .insert_unique(hash, (key, value), |entry| self.indexer.hash(&entry.0).0);
            }
        }

        if map.routing.merge_children(children, survivor).is_none() {
            debug_assert!(false, "merge of a locked branch must succeed");
            return false;
        }
        // Invalidate every transaction that routed to any of the merged
        // leaves, not just the survivor: the other seven are about to be
        // freed and reused.
        for &id in &children {
            map.bump_version(id);
        }
        drop(guards);

        // Free the seven non-surviving ids. The mask keeps transactions out
        // while the leaves are torn down.
        for &id in &children[1..] {
            map.free_ids(&[id]);
            // SAFETY: `seen` still holds `id` and the write guards are gone.
            unsafe { drop(map.take_leaf(id)) };
        }
        true
    }
}
