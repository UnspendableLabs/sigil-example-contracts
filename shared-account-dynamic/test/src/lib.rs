#[cfg(test)]
mod tests {
    use testlib::*;

    import!(
        name = "shared-account-dynamic",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
    );

    import!(
        name = "token",
        height = 0,
        tx_index = 0,
        path = "../token/contract/wit",
    );

    #[tokio::test]
    async fn test_contract() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[
                    ("shared-account-dynamic", &contract_bytes().await?),
                    ("token", &dep_contract_bytes("token").await?),
                    ("other-token", &dep_contract_bytes("token").await?),
                ])
                .build(),
        )
        .await?;

        let alice = "alice";
        let bob = "bob";
        let claire = "claire";
        let dara = "dara";

        token::mint(&runtime, alice, 100.into()).await?;

        let token_address = ContractAddress {
            name: "token".to_string(),
            height: 0,
            tx_index: 0,
        };

        let account_id = shared_account_dynamic::open(
            &runtime,
            alice,
            token_address.clone(),
            50.into(),
            vec![bob, dara],
        )
        .await??;

        let result = shared_account_dynamic::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(50.into()));

        let result = shared_account_dynamic::withdraw(
            &runtime,
            alice,
            ContractAddress {
                name: "other-token".to_string(),
                height: 0,
                tx_index: 0,
            },
            &account_id,
            50.into(),
        )
        .await?;
        assert_eq!(result, Err(Error::new("unauthorized")));

        shared_account_dynamic::deposit(
            &runtime,
            alice,
            token_address.clone(),
            &account_id,
            25.into(),
        )
        .await??;

        let result = shared_account_dynamic::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(75.into()));

        shared_account_dynamic::withdraw(
            &runtime,
            bob,
            token_address.clone(),
            &account_id,
            25.into(),
        )
        .await??;

        let result = shared_account_dynamic::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(50.into()));

        shared_account_dynamic::withdraw(
            &runtime,
            alice,
            token_address.clone(),
            &account_id,
            50.into(),
        )
        .await??;

        let result = shared_account_dynamic::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(0.into()));

        let result = shared_account_dynamic::withdraw(
            &runtime,
            bob,
            token_address.clone(),
            &account_id,
            1.into(),
        )
        .await?;
        assert_eq!(result, Err(Error::new("insufficient balance")));

        let result = shared_account_dynamic::withdraw(
            &runtime,
            claire,
            token_address.clone(),
            &account_id,
            1.into(),
        )
        .await?;
        assert_eq!(result, Err(Error::new("unauthorized")));

        let result = shared_account_dynamic::tenants(&runtime, &account_id).await?;
        assert_eq!(
            result,
            Some(vec![alice.to_string(), bob.to_string(), dara.to_string()])
        );

        Ok(())
    }
}
