use crate::tx_schema;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct User {
    pub first_name: String,
    pub last_name: String,
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
