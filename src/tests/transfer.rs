#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::types::types::{Funds, Transfer, User},
    };

    #[test]
    fn transfer() {
        let db: TxMap<User, Funds> = crate::tests::creators::creators::empty_typed_map();
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
        db.transaction()
            .update(tim.clone(), |_t, _f| {
                Some(Funds {
                    usd_and_cents: 150,
                    sterling_and_pence: 0,
                })
            })
            .into_transaction()
            .execute();
        let send_1_usd_from_bob_to_tim = db
            .transaction()
            .require("Has available funds", [tim.clone()], |[tim_funds]| {
                tim_funds.is_some_and(|f| f.usd_and_cents > 100)
            })
            .update(tim.clone(), |_t, tim_funds| {
                Some(Funds {
                    sterling_and_pence: tim_funds.unwrap().sterling_and_pence,
                    usd_and_cents: tim_funds.unwrap().usd_and_cents - 100,
                })
            })
            .update(bob.clone(), |_b, bob_funds| {
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
            send_1_usd_from_bob_to_tim.execute(),
            TxResult::Completed(())
        );
        assert_ne!(
            send_1_usd_from_bob_to_tim.execute(),
            TxResult::Completed(())
        );

        let send_x_usd_from_bob_to_tim = db
            .transaction()
            .with_param::<Transfer>()
            .require(
                "Has available funds",
                [tim.clone()],
                |[tim_funds], params| {
                    tim_funds.is_some_and(|f| f.usd_and_cents >= params.usd_and_cents)
                },
            )
            .insert_default_if_absent(bob.clone())
            .modify(bob.clone(), |_bob, funds, params| {
                funds.usd_and_cents -= params.usd_and_cents
            })
            .modify(tim.clone(), |_tim, funds, params| {
                funds.usd_and_cents += params.usd_and_cents
            })
            .get_all([bob.clone(), tim.clone()], |_user, funds| {
                funds.usd_and_cents
            })
            .into_transaction();
        assert_eq!(
            send_x_usd_from_bob_to_tim.execute(&Transfer { usd_and_cents: 40 }),
            TxResult::Completed(vec![Some(60), Some(90)])
        );
        assert_ne!(
            send_x_usd_from_bob_to_tim.execute(&Transfer { usd_and_cents: 20 }),
            TxResult::Completed(vec![Some(40), Some(60)])
        );

        let add_100_usd_to_bob_if_exists = db
            .transaction()
            .modify(bob.clone(), |_b, bob_funds| {
                bob_funds.usd_and_cents += 100;
            })
            .into_transaction();
        assert_eq!(
            add_100_usd_to_bob_if_exists.execute(),
            TxResult::Completed(())
        );
        assert_eq!(
            add_100_usd_to_bob_if_exists.execute(),
            TxResult::Completed(())
        );

        let add_123_to_pam = db
            .transaction()
            .insert_default_if_absent(pam.clone())
            .modify(pam.clone(), |_p, pam_funds| {
                pam_funds.usd_and_cents += 123;
            })
            .get(pam.clone(), |_user, funds| funds.usd_and_cents)
            .into_transaction();
        assert_eq!(add_123_to_pam.execute(), TxResult::Completed(Some(123)));
        assert_eq!(add_123_to_pam.execute(), TxResult::Completed(Some(246)));
    }
}
