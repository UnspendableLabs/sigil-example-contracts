#[cfg(test)]
mod tests {
    use testlib::*;

    import!(
        name = "proxy",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
    );

    import!(
        mod_name = "shared_account_dynamic_proxied",
        name = "proxy",
        height = 0,
        tx_index = 0,
        path = "../shared-account-dynamic/contract/wit",
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
                    ("token", &dep_contract_bytes("token").await?),
                    (
                        "shared-account-dynamic",
                        &dep_contract_bytes("shared-account-dynamic").await?,
                    ),
                    ("proxy", &contract_bytes().await?),
                ])
                .build(),
        )
        .await?;

        let alice = "alice";
        let bob = "bob";
        let claire = "claire";
        let dara = "dara";

        let result = proxy::get_contract_address(&runtime).await?;
        assert_eq!(result, None);

        let address = ContractAddress {
            name: "shared-account-dynamic".to_string(),
            height: 0,
            tx_index: 0,
        };
        proxy::set_contract_address(&runtime, alice, address.clone()).await?;

        let result = proxy::get_contract_address(&runtime).await?;
        assert_eq!(result, Some(address));

        token::mint(&runtime, alice, 100.into()).await?;

        let token_address = ContractAddress {
            name: "token".to_string(),
            height: 0,
            tx_index: 0,
        };

        let account_id = shared_account_dynamic_proxied::open(
            &runtime,
            alice,
            token_address.clone(),
            50.into(),
            vec![bob, dara],
        )
        .await??;

        let result = shared_account_dynamic_proxied::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(50.into()));

        let result = shared_account_dynamic_proxied::withdraw(
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

        shared_account_dynamic_proxied::deposit(
            &runtime,
            alice,
            token_address.clone(),
            &account_id,
            25.into(),
        )
        .await??;

        let result = shared_account_dynamic_proxied::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(75.into()));

        shared_account_dynamic_proxied::withdraw(
            &runtime,
            bob,
            token_address.clone(),
            &account_id,
            25.into(),
        )
        .await??;

        let result = shared_account_dynamic_proxied::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(50.into()));

        shared_account_dynamic_proxied::withdraw(
            &runtime,
            alice,
            token_address.clone(),
            &account_id,
            50.into(),
        )
        .await??;

        let result = shared_account_dynamic_proxied::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(0.into()));

        let result = shared_account_dynamic_proxied::withdraw(
            &runtime,
            bob,
            token_address.clone(),
            &account_id,
            1.into(),
        )
        .await?;
        assert_eq!(result, Err(Error::new("insufficient balance")));

        let result = shared_account_dynamic_proxied::withdraw(
            &runtime,
            claire,
            token_address.clone(),
            &account_id,
            1.into(),
        )
        .await?;
        assert_eq!(result, Err(Error::new("unauthorized")));

        let result = shared_account_dynamic_proxied::tenants(&runtime, &account_id).await?;
        assert_eq!(
            result,
            Some(vec![alice.to_string(), bob.to_string(), dara.to_string()])
        );

        Ok(())
    }
}
