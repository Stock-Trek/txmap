use crate::{
    custodian::Custodian,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{guard::Guard, ops::op_trait::OpTrait, params::TxKeys},
    result::TxResult,
};
use std::hash::Hash;

pub struct PreparedTransaction<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, L, KEYS, PARAMS, STATE> + 'tx>>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> PreparedTransaction<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    #[must_use]
    pub fn execute<RAW>(&self, keys: RAW, params: PARAMS) -> TxResult<STATE>
    where
        RAW: TxKeys<K, KEYS>,
    {
        let keys = keys.into_indexed(self.custodian.shard_count);
        let mut total_read_bitmask = BitMask::ZERO;
        let mut total_write_bitmask = BitMask::ZERO;

        // get all bitmasks
        for guard in self.guards.iter() {
            total_read_bitmask |= guard.read_bitmask(&keys);
        }
        for op in self.ops.iter() {
            let (read_bitmask, write_bitmask) = op.read_write_bitmasks(&keys);
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
            if !guard.is_condition_met::<L>(&mut lock_guards, &keys, &params, &mut state) {
                return TxResult::RequirementNotMet(i, guard.name.clone());
            }
        }
        for op in self.ops.iter() {
            op.apply(&mut lock_guards, &keys, &params, &mut state);
        }
        TxResult::Completed(state)
    }
}
