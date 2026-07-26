use crate::{
    custodian::Custodian,
    guard::Guard,
    lock_policies::lock_policy::LockPolicy,
    ops::{
        get_op::GetOp, insert_default_if_absent_op::InsertDefaultIfAbsentOp,
        insert_default_op::InsertDefaultOp, insert_with_if_absent_op::InsertWithIfAbsentOp,
        insert_with_op::InsertWithOp, modify_op::ModifyOp, move_value_op::MoveValueOp,
        op_trait::OpTrait, remove_op::RemoveOp, remove_where_op::RemoveWhereOp,
        swap_value_op::SwapValueOp, update_op::UpdateOp,
    },
    params::{TxKey, TxKeySelector},
    transaction::Transaction,
};
use std::{hash::Hash, marker::PhantomData};

// ─── Type-state markers ───────────────────────────────────────────────────────

/// State: `require()` and all ops are available, but not `into_transaction()`.
pub struct BuilderPhase;

/// State: all ops and `into_transaction()` are available.
pub struct BuildablePhase;

// ─── Single builder struct ────────────────────────────────────────────────────

pub struct TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PHASE = BuilderPhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, L, KEYS, PARAMS, STATE> + 'tx>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Operations available in any phase (BuilderPhase or BuildablePhase)
//  Each operation transitions to BuildablePhase.
// ═══════════════════════════════════════════════════════════════════════════════

impl<'tx, K, V, L, KEYS, PARAMS, STATE, PHASE>
    TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PHASE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        get: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase> {
        let op = GetOp {
            key_selector: Box::new(key_selector),
            get: Box::new(get),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn insert_default(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultOp {
            key_selector: Box::new(key_selector),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn insert_default_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultIfAbsentOp {
            key_selector: Box::new(key_selector),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn insert_with(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
    {
        let op = InsertWithOp {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn insert_with_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
    {
        let op = InsertWithIfAbsentOp {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn modify(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase> {
        let op = ModifyOp {
            key_selector: Box::new(key_selector),
            mutate: Box::new(mutate),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn move_value(
        mut self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
    {
        let op = MoveValueOp {
            key_selector_from: Box::new(key_selector_from),
            key_selector_to: Box::new(key_selector_to),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn remove(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        on_remove: impl Fn(Option<(K, V)>, &PARAMS, &mut STATE) + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase> {
        let op = RemoveOp {
            key_selector: Box::new(key_selector),
            on_remove: Box::new(on_remove),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn remove_where(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase> {
        let op = RemoveWhereOp {
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn swap_value(
        mut self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
    {
        let op = SwapValueOp {
            key_selector_a: Box::new(key_selector_a),
            key_selector_b: Box::new(key_selector_b),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    pub fn update(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
    where
        K: Clone,
    {
        let op = UpdateOp {
            key_selector: Box::new(key_selector),
            transform: Box::new(transform),
        };
        self.ops.push(Box::new(op));
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  BuilderPhase – require() only
// ═══════════════════════════════════════════════════════════════════════════════

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuilderPhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    /// Add a guard (requirement) that must be satisfied before the transaction
    /// executes.
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuilderPhase> {
        let guard = Guard {
            name: name.as_ref().into(),
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
            _phantom: PhantomData,
        };
        self.guards.push(guard);
        TxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  BuildablePhase – into_transaction() only
// ═══════════════════════════════════════════════════════════════════════════════

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuildablePhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    /// Consume the builder and produce a [`Transaction`] ready for execution.
    #[must_use]
    pub fn into_transaction(self) -> Transaction<'tx, K, V, L, KEYS, PARAMS, STATE> {
        Transaction {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
        }
    }
}
