use crate::tx_schema;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct User {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Funds {
    pub usd_and_cents: u64,
    pub sterling_and_pence: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Counter {
    pub value: u64,
}

tx_schema! {
    Transfer,
    keys: [
        from,
        to,
    ],
    params: {
        amount: u64,
    },
    state: {
        results: Vec<Option<u64>>,
    },
    value: Funds,
}

tx_schema! {
    ConcurrentTransfer,
    keys: [
        from,
        to,
    ],
    params: {
        amount: u64,
    },
    state: {
    },
    value: u64,
}

tx_schema! {
    RemoveMultiple,
    keys: [
        a,
        b,
    ],
    params: {
    },
    state: {
        user: Vec<Option<String>>,
    },
    value: u64,
}

tx_schema! {
    Increment,
    keys: [
        k,
    ],
    params: {
    },
    state: {
    },
    value: u64,
}

tx_schema! {
    GetOne,
    keys: [
        key,
    ],
    params: {
    },
    state: {
        result: Option<u64>,
    },
    value: u64,
}

tx_schema! {
    GetOneParamU64,
    keys: [
        key,
    ],
    params: {
        param: u64,
    },
    state: {
        result: Option<u64>,
    },
    value: u64,
}

tx_schema! {
    GetOneParamString,
    keys: [
        key,
    ],
    params: {
        param: String,
    },
    state: {
        result: Option<String>,
    },
    value: String,
}

tx_schema! {
    GetTwo,
    keys: [
        a,
        b,
    ],
    params: {
    },
    state: {
        result_a: Option<u64>,
        result_b: Option<u64>,
    },
    value: u64,
}

tx_schema! {
    GetTwoParam,
    keys: [
        a,
        b,
    ],
    params: {
        _p: (),
    },
    state: {
        result_a: Option<u64>,
        result_b: Option<u64>,
    },
    value: u64,
}

tx_schema! {
    GetTwoParamU64,
    keys: [
        a,
        b,
    ],
    params: {
        param: u64,
    },
    state: {
        result_a: Option<u64>,
        result_b: Option<u64>,
    },
    value: u64,
}

tx_schema! {
    GetThree,
    keys: [
        a,
        b,
        c,
    ],
    params: {
    },
    state: {
        results: Vec<Option<u64>>,
    },
    value: u64,
}

tx_schema! {
    GetVecParam,
    keys: [
        a,
        b,
    ],
    params: {
        param: Vec<u64>,
    },
    state: {
        results: Vec<Option<u64>>,
    },
    value: u64,
}

tx_schema! {
    GetCounter,
    keys: [
        key,
    ],
    params: {
    },
    state: {
        result: Option<u64>,
    },
    value: Counter,
}

tx_schema! {
    SetOne,
    keys: [
        key,
    ],
    params: {
    },
    state: {
    },
    value: u64,
}

// Exercises the optional `lock_policy`/`hasher` clauses of `tx_schema!`.
tx_schema! {
    RwLockGetOne,
    keys: [
        key,
    ],
    params: {
    },
    state: {
    },
    value: u64,
    lock_policy: RwLockPolicy,
}

tx_schema! {
    HashedGetOne,
    keys: [
        key,
    ],
    params: {
    },
    state: {
    },
    value: u64,
    hasher: crate::hasher::DefaultBuildHasher,
}

tx_schema! {
    RwLockHashedGetOne,
    keys: [
        key,
    ],
    params: {
    },
    state: {
    },
    value: u64,
    lock_policy: RwLockPolicy,
    hasher: crate::hasher::DefaultBuildHasher,
}
