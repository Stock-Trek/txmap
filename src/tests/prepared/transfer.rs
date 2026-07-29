use crate::{
    prelude::*,
    tests::{creators::*, types::*},
};

#[test]
fn transfer() {
    let db: TxMap<User, Funds> = empty_typed_map();
    let bob = User {
        first_name: "Bob".into(),
        last_name: "Bobson".into(),
    };
    let tim = User {
        first_name: "Tim".into(),
        last_name: "Timson".into(),
    };
    let pam = User {
        first_name: "Pam".into(),
        last_name: "Pamson".into(),
    };

    // Initial setup: give Tim 150 USD
    let _ = db
        .prepared_tx(&Transfer::SCHEMA)
        .insert_with(Transfer::to, |_u, _p, _s| Funds {
            usd_and_cents: 150,
            sterling_and_pence: 0,
        })
        .into_transaction()
        .execute(
            TransferKeys {
                from: tim.clone(),
                to: tim.clone(),
            },
            TransferParams { amount: 0 },
        );

    // Send 1 USD from Tim to Bob (using update, no params)
    let send_1_usd = db
        .prepared_tx(&Transfer::SCHEMA)
        .require("Has available funds", Transfer::from, |_k, v, _p, _s| {
            v.is_some_and(|f| f.usd_and_cents > 100)
        })
        .update(Transfer::from, |_t, tim_funds, _p, _s| {
            tim_funds.map(|f| Funds {
                sterling_and_pence: f.sterling_and_pence,
                usd_and_cents: f.usd_and_cents - 100,
            })
        })
        .update(Transfer::to, |_b, bob_funds, _p, _s| {
            Some(bob_funds.map_or(
                Funds {
                    usd_and_cents: 100,
                    sterling_and_pence: 0,
                },
                |f| Funds {
                    usd_and_cents: f.usd_and_cents + 100,
                    sterling_and_pence: f.sterling_and_pence,
                },
            ))
        })
        .into_transaction();
    assert_eq!(
        send_1_usd.execute(
            TransferKeys {
                from: tim.clone(),
                to: bob.clone()
            },
            TransferParams { amount: 0 }
        ),
        TxResult::Completed(TransferState { results: vec![] })
    );
    assert_ne!(
        send_1_usd.execute(
            TransferKeys {
                from: tim.clone(),
                to: bob.clone()
            },
            TransferParams { amount: 0 }
        ),
        TxResult::Completed(TransferState { results: vec![] })
    );

    // Send X USD from Bob to Tim (parameterized with get verification)
    let send_x_usd = db
        .prepared_tx(&Transfer::SCHEMA)
        .require("Has available funds", Transfer::from, |_k, v, p, _s| {
            v.is_some_and(|f| f.usd_and_cents >= p.amount)
        })
        .insert_with_if_absent(Transfer::to, |_k, _p, _s| Funds::default())
        .modify(Transfer::from, |_bob, funds, p, _s| {
            funds.usd_and_cents -= p.amount
        })
        .modify(Transfer::to, |_tim, funds, p, _s| {
            funds.usd_and_cents += p.amount
        })
        .get(Transfer::from, |_user, funds, _p, s| {
            s.results.push(funds.map(|f| f.usd_and_cents));
        })
        .get(Transfer::to, |_user, funds, _p, s| {
            s.results.push(funds.map(|f| f.usd_and_cents));
        })
        .into_transaction();
    assert_eq!(
        send_x_usd.execute(
            TransferKeys {
                from: bob.clone(),
                to: tim.clone()
            },
            TransferParams { amount: 40 }
        ),
        TxResult::Completed(TransferState {
            results: vec![Some(60), Some(90)]
        })
    );

    // Add 100 USD to Bob (modify existing)
    let add_100_usd_to_bob = db
        .prepared_tx(&Transfer::SCHEMA)
        .modify(Transfer::from, |_b, funds, _p, _s| {
            funds.usd_and_cents += 100;
        })
        .into_transaction();
    assert_eq!(
        add_100_usd_to_bob.execute(
            TransferKeys {
                from: bob.clone(),
                to: bob.clone()
            },
            TransferParams { amount: 0 }
        ),
        TxResult::Completed(TransferState { results: vec![] })
    );
    assert_eq!(
        add_100_usd_to_bob.execute(
            TransferKeys {
                from: bob.clone(),
                to: bob.clone()
            },
            TransferParams { amount: 0 }
        ),
        TxResult::Completed(TransferState { results: vec![] })
    );

    // Add 123 to Pam (insert_default_if_absent + modify + verify with get)
    let add_123_to_pam = db
        .prepared_tx(&Transfer::SCHEMA)
        .insert_with_if_absent(Transfer::from, |_k, _p, _s| Funds::default())
        .modify(Transfer::from, |_p, funds, _p2, _s| {
            funds.usd_and_cents += 123;
        })
        .get(Transfer::from, |_user, funds, _p, s| {
            s.results.push(funds.map(|f| f.usd_and_cents));
        })
        .into_transaction();
    assert_eq!(
        add_123_to_pam.execute(
            TransferKeys {
                from: pam.clone(),
                to: pam.clone()
            },
            TransferParams { amount: 0 }
        ),
        TxResult::Completed(TransferState {
            results: vec![Some(123)]
        })
    );
    assert_eq!(
        add_123_to_pam.execute(
            TransferKeys {
                from: pam.clone(),
                to: pam.clone()
            },
            TransferParams { amount: 0 }
        ),
        TxResult::Completed(TransferState {
            results: vec![Some(246)]
        })
    );
}
