use crate::{
    custodian::Custodian,
    immediate::{guard::Guard, ops::op_trait::OpTrait},
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    result::TxResult,
};
use std::hash::Hash;

pub struct ImmediateTransaction<'tx, K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, L, STATE> + 'tx>>,
}

impl<'tx, K, V, L, STATE> ImmediateTransaction<'tx, K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    #[must_use]
    pub fn execute(&self) -> TxResult<STATE> {
        let mut total_read_bitmask = BitMask::ZERO;
        let mut total_write_bitmask = BitMask::ZERO;

        // get all bitmasks
        for guard in self.guards.iter() {
            total_read_bitmask |= guard.read_bitmask();
        }
        for op in self.ops.iter() {
            let (read_bitmask, write_bitmask) = op.read_write_bitmasks();
            total_read_bitmask |= read_bitmask;
            total_write_bitmask |= write_bitmask;
        }
        // ensure locks are either read or write, not both
        total_read_bitmask &= !total_write_bitmask;

        let mut lock_guards = self
            .custodian
            .lock_guards(total_read_bitmask, total_write_bitmask);
        let mut state = STATE::default();
        for (i, guard) in self.guards.iter().enumerate() {
            if !guard.is_condition_met::<L>(&mut lock_guards, &mut state) {
                return TxResult::RequirementNotMet(i, guard.name.clone());
            }
        }
        for op in self.ops.iter() {
            op.apply(&mut lock_guards, &mut state);
        }
        TxResult::Completed(state)
    }
}
