use crate::{
    prelude::*,
    tests::{creators::*, data::*, types::*},
};

#[test]
fn param_map_op() {
    let map = map_alice(10);
    let factor = 3u64;
    let result = map
        .immediate_tx::<GetOneParamU64State>()
        .update(ALICE.into(), move |_k, v, s| {
            let r = v.map(|x| x * factor);
            s.result = r;
            r
        })
        .execute();
    assert_eq!(
        result,
        TxResult::Completed {
            state: GetOneParamU64State { result: Some(30) }
        }
    );
}
