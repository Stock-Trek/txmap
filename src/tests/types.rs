use crate::tx_schema;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct User {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct Funds {
    pub usd_and_cents: u64,
    pub sterling_and_pence: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct Counter {
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
}

// Schema for one-key operations with u64 values, captures one result
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
}

// Schema for one-key operations with u64 param, captures one result
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
}

// Schema for one-key operations with String param, captures one result
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
}

// Schema for two-key operations, captures two results
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
}

// Schema for two-key operations with () param, captures two results
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
}

// Schema for two-key operations with u64 param, captures two results
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
}

// Schema for three-key operations, captures results in a vec
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
}

// Schema for two-key operations with Vec<u64> param
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
}

// Schema for one-key Counter operations, captures counter value
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
}

// Schema for single-key operations with u64 value and no state needed
tx_schema! {
    SetOne,
    keys: [
        key,
    ],
    params: {
    },
    state: {
    },
}
