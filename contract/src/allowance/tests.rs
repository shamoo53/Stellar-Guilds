#[cfg(test)]
mod tests {
    use crate::allowance::management;
    use crate::allowance::types::{AllowanceError, AllowanceOperation};
    use crate::StellarGuildsContract;
    use crate::StellarGuildsContractClient;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, Env};

    fn setup<'a>() -> (
        Env,
        Address,
        Address,
        StellarGuildsContractClient<'a>,
        Address,
    ) {
        let env = Env::default();
        env.budget().reset_unlimited();
        env.mock_all_auths();

        set_ledger_timestamp(&env, 1_000);

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let owner = Address::generate(&env);
        let spender = Address::generate(&env);
        (env, owner, spender, client, contract_id)
    }

    fn set_ledger_timestamp(env: &Env, timestamp: u64) {
        env.ledger().set(LedgerInfo {
            timestamp,
            protocol_version: 20,
            sequence_number: 0,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 1_000_000,
        });
    }

    // ── Happy Path ───────────────────────────────────────────────────────

    #[test]
    fn test_approve_and_spend() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(
            &owner,
            &spender,
            &None,
            &1000,
            &0,
            &AllowanceOperation::Any,
        );

        let allowance = client.get_token_allowance(&owner, &spender, &None);
        assert_eq!(allowance.amount, 1000);
        assert_eq!(allowance.spent, 0);
        assert_eq!(allowance.remaining(), 1000);
        assert_eq!(allowance.operation, AllowanceOperation::Any);

        let spend_result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                300,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(spend_result, Ok(()));

        let allowance = client.get_token_allowance(&owner, &spender, &None);
        assert_eq!(allowance.spent, 300);
        assert_eq!(allowance.remaining(), 700);
    }

    // ── Expiration ───────────────────────────────────────────────────────

    #[test]
    fn test_expired_allowance_blocks_spend() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(
            &owner,
            &spender,
            &None,
            &500,
            &2000,
            &AllowanceOperation::Any,
        );

        set_ledger_timestamp(&env, 3000);

        let result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                100,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result, Err(AllowanceError::Expired));
    }

    #[test]
    fn test_spend_before_expiry_works() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(
            &owner,
            &spender,
            &None,
            &500,
            &5000,
            &AllowanceOperation::Any,
        );

        set_ledger_timestamp(&env, 4999);

        let result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                100,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result, Ok(()));
    }

    // ── Revocation ───────────────────────────────────────────────────────

    #[test]
    fn test_revoke_blocks_spend() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &500, &0, &AllowanceOperation::Any);

        client.revoke_token_allowance(&owner, &spender, &None);

        let result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                100,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result, Err(AllowanceError::NotFound));
    }

    #[test]
    #[should_panic(expected = "allowance not found")]
    fn test_revoke_nonexistent_returns_error() {
        let (_env, owner, spender, client, _) = setup();
        client.revoke_token_allowance(&owner, &spender, &None);
    }

    // ── Over-spend ───────────────────────────────────────────────────────

    #[test]
    fn test_overspend_blocked() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &100, &0, &AllowanceOperation::Any);

        let result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                101,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result, Err(AllowanceError::InsufficientAllowance));
    }

    #[test]
    fn test_exact_spend_works() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &100, &0, &AllowanceOperation::Any);

        let result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                100,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result, Ok(()));

        let result2 = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                1,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result2, Err(AllowanceError::InsufficientAllowance));
    }

    // ── Per-Operation Filtering ──────────────────────────────────────────

    #[test]
    fn test_operation_filter_blocks_wrong_type() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(
            &owner,
            &spender,
            &None,
            &500,
            &0,
            &AllowanceOperation::Withdrawal,
        );

        let result = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                100,
                &AllowanceOperation::MilestonePayment,
            )
        });
        assert_eq!(result, Err(AllowanceError::OperationNotPermitted));

        let result2 = env.as_contract(&contract_id, || {
            management::spend(
                &env,
                &spender,
                &owner,
                &None,
                100,
                &AllowanceOperation::Withdrawal,
            )
        });
        assert_eq!(result2, Ok(()));
    }

    #[test]
    fn test_any_operation_permits_all() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &500, &0, &AllowanceOperation::Any);

        assert_eq!(
            env.as_contract(&contract_id, || management::spend(
                &env,
                &spender,
                &owner,
                &None,
                50,
                &AllowanceOperation::Withdrawal
            )),
            Ok(())
        );
        assert_eq!(
            env.as_contract(&contract_id, || management::spend(
                &env,
                &spender,
                &owner,
                &None,
                50,
                &AllowanceOperation::BountyFunding
            )),
            Ok(())
        );
        assert_eq!(
            env.as_contract(&contract_id, || management::spend(
                &env,
                &spender,
                &owner,
                &None,
                50,
                &AllowanceOperation::MilestonePayment
            )),
            Ok(())
        );
        assert_eq!(
            env.as_contract(&contract_id, || management::spend(
                &env,
                &spender,
                &owner,
                &None,
                50,
                &AllowanceOperation::Escrow
            )),
            Ok(())
        );
    }

    // ── Increase / Decrease ──────────────────────────────────────────────

    #[test]
    fn test_increase_allowance() {
        let (_env, owner, spender, client, _) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &100, &0, &AllowanceOperation::Any);
        client.increase_token_allowance(&owner, &spender, &None, &50);

        let allowance = client.get_token_allowance(&owner, &spender, &None);
        assert_eq!(allowance.amount, 150);
        assert_eq!(allowance.remaining(), 150);
    }

    #[test]
    fn test_decrease_allowance() {
        let (_env, owner, spender, client, _) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &200, &0, &AllowanceOperation::Any);
        client.decrease_token_allowance(&owner, &spender, &None, &80);

        let allowance = client.get_token_allowance(&owner, &spender, &None);
        assert_eq!(allowance.amount, 120);
    }

    #[test]
    fn test_decrease_floors_at_spent() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &200, &0, &AllowanceOperation::Any);

        env.as_contract(&contract_id, || {
            management::spend(&env, &spender, &owner, &None, 150, &AllowanceOperation::Any)
        })
        .unwrap();

        client.decrease_token_allowance(&owner, &spender, &None, &100);

        let allowance = client.get_token_allowance(&owner, &spender, &None);
        assert_eq!(allowance.amount, 150); // Floored
        assert_eq!(allowance.remaining(), 0);
    }

    // ── Index Queries ────────────────────────────────────────────────────

    #[test]
    fn test_list_by_owner() {
        let (env, owner, spender, client, _) = setup();
        let spender2 = Address::generate(&env);

        client.approve_token_allowance(&owner, &spender, &None, &100, &0, &AllowanceOperation::Any);
        client.approve_token_allowance(
            &owner,
            &spender2,
            &None,
            &200,
            &0,
            &AllowanceOperation::Withdrawal,
        );

        let list = client.get_owner_allowances(&owner);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_list_by_spender() {
        let (env, owner, spender, client, _) = setup();
        let owner2 = Address::generate(&env);

        client.approve_token_allowance(&owner, &spender, &None, &100, &0, &AllowanceOperation::Any);
        client.approve_token_allowance(
            &owner2,
            &spender,
            &None,
            &200,
            &0,
            &AllowanceOperation::Withdrawal,
        );

        let list = client.get_spender_allowances(&spender);
        assert_eq!(list.len(), 2);
    }

    // ── Invalid Amount ───────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "invalid amount")]
    fn test_approve_zero_amount_fails() {
        let (_env, owner, spender, client, _) = setup();
        client.approve_token_allowance(&owner, &spender, &None, &0, &0, &AllowanceOperation::Any);
    }

    #[test]
    #[should_panic(expected = "invalid amount")]
    fn test_approve_negative_amount_fails() {
        let (_env, owner, spender, client, _) = setup();
        client.approve_token_allowance(&owner, &spender, &None, &-50, &0, &AllowanceOperation::Any);
    }

    // ── Approve Replaces Existing ────────────────────────────────────────

    #[test]
    fn test_approve_replaces_existing() {
        let (env, owner, spender, client, contract_id) = setup();

        client.approve_token_allowance(&owner, &spender, &None, &100, &0, &AllowanceOperation::Any);

        env.as_contract(&contract_id, || {
            management::spend(&env, &spender, &owner, &None, 30, &AllowanceOperation::Any)
        })
        .unwrap();

        client.approve_token_allowance(
            &owner,
            &spender,
            &None,
            &200,
            &0,
            &AllowanceOperation::Withdrawal,
        );

        let allowance = client.get_token_allowance(&owner, &spender, &None);
        assert_eq!(allowance.amount, 200);
        assert_eq!(allowance.spent, 0); // Reset
        assert_eq!(allowance.operation, AllowanceOperation::Withdrawal);
    }
}
