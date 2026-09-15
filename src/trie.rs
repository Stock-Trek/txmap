//! Lock-free routing trie.
//!
//! The trie maps a hash prefix to a leaf id. It consumes [`BITS_PER_LEVEL`]
//! bits per level and every branch has [`FANOUT`] children. Reads take no
//! locks; splits and merges atomically swap the affected node. Branch nodes
//! are reclaimed with epoch-based garbage collection, while leaf nodes are
//! plain ids so leaf values never move.

use crossbeam_epoch::{self as epoch, Atomic, Guard, Owned, Shared};
use std::sync::atomic::Ordering;

/// Number of bits consumed by each trie level.
pub(crate) const BITS_PER_LEVEL: u32 = 3;
/// Number of children per branch node (`1 << BITS_PER_LEVEL`).
pub(crate) const FANOUT: usize = 1 << BITS_PER_LEVEL;

/// A routing trie node.
pub(crate) enum Node {
    /// Terminal node pointing at a leaf id (`0..=127`).
    Leaf(u8),
    /// Internal node with one child per hash prefix.
    Branch([Atomic<Node>; FANOUT]),
}

impl Node {
    fn branch(children: [u8; FANOUT]) -> Self {
        Node::Branch(std::array::from_fn(|i| {
            Atomic::new(Node::Leaf(children[i]))
        }))
    }
}

/// Maps hash prefixes to leaf ids.
pub(crate) struct RoutingTrie {
    root: Atomic<Node>,
}

impl RoutingTrie {
    /// Creates a trie with a single leaf.
    pub(crate) fn new(leaf_id: u8) -> Self {
        Self {
            root: Atomic::new(Node::Leaf(leaf_id)),
        }
    }

    /// Returns the child index for `hash` at the given `level`.
    ///
    /// Routing consumes the *high* bits of the hash, walking from the top
    /// down. Hashbrown stores a 7-bit control byte in the low bits, so
    /// keeping routing in the high bits leaves per-leaf tables with full tag
    /// entropy.
    #[inline]
    pub(crate) fn child_index(hash: u64, level: u32) -> usize {
        let shift = 64 - (level + 1) * BITS_PER_LEVEL;
        ((hash >> shift) & (FANOUT as u64 - 1)) as usize
    }

    /// Routes `hash` to a leaf id.
    #[inline]
    pub(crate) fn route(&self, hash: u64) -> u8 {
        let guard = epoch::pin();
        let mut node = self.root.load(Ordering::Acquire, &guard);
        let mut level = 0;
        loop {
            // SAFETY: the epoch guard keeps every node reachable from `node`
            // alive for the duration of this traversal.
            match unsafe { node.deref() } {
                Node::Leaf(id) => return *id,
                Node::Branch(children) => {
                    node = children[Self::child_index(hash, level)].load(Ordering::Acquire, &guard);
                    level += 1;
                }
            }
        }
    }

    /// Returns the level at which `leaf_id` currently sits, if present.
    pub(crate) fn level_of(&self, leaf_id: u8) -> Option<u32> {
        let guard = epoch::pin();
        let root = self.root.load(Ordering::Acquire, &guard);
        level_in(root, leaf_id, 0, &guard)
    }

    /// Replaces the leaf `leaf_id` with a branch of `children`.
    ///
    /// The child at index `i` becomes reachable when the next
    /// [`BITS_PER_LEVEL`] bits equal `i`. Returns the level of the replaced
    /// leaf, or `None` if the leaf was not found or the trie changed
    /// concurrently.
    pub(crate) fn split_leaf(&self, leaf_id: u8, children: [u8; FANOUT]) -> Option<u32> {
        let guard = epoch::pin();
        self.split_in_slot(&self.root, leaf_id, children, 0, &guard)
    }

    fn split_in_slot(
        &self,
        slot: &Atomic<Node>,
        leaf_id: u8,
        children: [u8; FANOUT],
        level: u32,
        guard: &Guard,
    ) -> Option<u32> {
        let node = slot.load(Ordering::Acquire, guard);
        match unsafe { node.deref() } {
            Node::Leaf(id) => {
                if *id != leaf_id {
                    return None;
                }
                let replacement = Owned::new(Node::branch(children));
                if slot
                    .compare_exchange(
                        node,
                        replacement,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                        guard,
                    )
                    .is_ok()
                {
                    Some(level)
                } else {
                    None
                }
            }
            Node::Branch(children_atomics) => {
                for child in children_atomics {
                    if let Some(found) =
                        self.split_in_slot(child, leaf_id, children, level + 1, guard)
                    {
                        return Some(found);
                    }
                }
                None
            }
        }
    }

    /// Returns the level and child leaf ids of the shallowest branch whose
    /// children are all leaves and which does not contain `exclude`.
    ///
    /// A breadth-first search finds the branch that frees the most routing
    /// depth. Passing `Some(leaf)` lets an adaptive merge free capacity
    /// without disturbing the hot leaf that is about to be split.
    pub(crate) fn mergeable_branch(&self, exclude: Option<u8>) -> Option<(u32, [u8; FANOUT])> {
        let guard = epoch::pin();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((self.root.load(Ordering::Acquire, &guard), 0u32));
        while let Some((node, level)) = queue.pop_front() {
            match unsafe { node.deref() } {
                Node::Leaf(_) => continue,
                Node::Branch(children) => {
                    let mut ids = [0u8; FANOUT];
                    let mut all_leaves = true;
                    for (i, child) in children.iter().enumerate() {
                        let loaded = child.load(Ordering::Acquire, &guard);
                        match unsafe { loaded.deref() } {
                            Node::Leaf(id) => ids[i] = *id,
                            Node::Branch(_) => all_leaves = false,
                        }
                    }
                    if all_leaves && exclude.is_none_or(|excluded| !ids.contains(&excluded)) {
                        return Some((level, ids));
                    }
                    for child in children {
                        let loaded = child.load(Ordering::Acquire, &guard);
                        if matches!(unsafe { loaded.deref() }, Node::Branch(_)) {
                            queue.push_back((loaded, level + 1));
                        }
                    }
                }
            }
        }
        None
    }

    /// Replaces a branch whose children are exactly `children` with
    /// `Leaf(survivor)`.
    pub(crate) fn merge_children(&self, children: [u8; FANOUT], survivor: u8) -> Option<u32> {
        let guard = epoch::pin();
        self.merge_in_slot(&self.root, children, survivor, 0, &guard)
    }

    fn merge_in_slot(
        &self,
        slot: &Atomic<Node>,
        expected: [u8; FANOUT],
        survivor: u8,
        level: u32,
        guard: &Guard,
    ) -> Option<u32> {
        let node = slot.load(Ordering::Acquire, guard);
        match unsafe { node.deref() } {
            Node::Leaf(_) => None,
            Node::Branch(children) => {
                let mut ids = [0u8; FANOUT];
                let mut all_leaves = true;
                for (i, child) in children.iter().enumerate() {
                    let loaded = child.load(Ordering::Acquire, guard);
                    match unsafe { loaded.deref() } {
                        Node::Leaf(id) => ids[i] = *id,
                        Node::Branch(_) => {
                            all_leaves = false;
                            break;
                        }
                    }
                }
                if all_leaves && same_children(ids, expected) {
                    let replacement = Owned::new(Node::Leaf(survivor));
                    if slot
                        .compare_exchange(
                            node,
                            replacement,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                            guard,
                        )
                        .is_ok()
                    {
                        return Some(level);
                    }
                }
                for child in children {
                    if let Some(found) =
                        self.merge_in_slot(child, expected, survivor, level + 1, guard)
                    {
                        return Some(found);
                    }
                }
                None
            }
        }
    }
}

fn same_children(mut a: [u8; FANOUT], mut b: [u8; FANOUT]) -> bool {
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

fn level_in(node: Shared<Node>, leaf_id: u8, level: u32, guard: &Guard) -> Option<u32> {
    // SAFETY: the caller holds an epoch guard covering `node`.
    match unsafe { node.deref() } {
        Node::Leaf(id) => (*id == leaf_id).then_some(level),
        Node::Branch(children) => {
            for child in children {
                let loaded = child.load(Ordering::Acquire, guard);
                if let Some(found) = level_in(loaded, leaf_id, level + 1, guard) {
                    return Some(found);
                }
            }
            None
        }
    }
}

impl Drop for RoutingTrie {
    fn drop(&mut self) {
        let guard = epoch::pin();
        let root = self.root.load(Ordering::Relaxed, &guard);
        if !root.is_null() {
            // SAFETY: `&mut self` guarantees no concurrent readers, so the
            // node can be taken and dropped recursively.
            drop_node(unsafe { root.into_owned() }, &guard);
        }
    }
}

fn drop_node(node: Owned<Node>, guard: &Guard) {
    let boxed = node.into_box();
    if let Node::Branch(children) = boxed.as_ref() {
        for child in children {
            let loaded = child.load(Ordering::Relaxed, guard);
            if !loaded.is_null() {
                // SAFETY: no other thread can reach these nodes while the
                // owning `RoutingTrie` is being dropped.
                drop_node(unsafe { loaded.into_owned() }, guard);
            }
        }
    }
}
