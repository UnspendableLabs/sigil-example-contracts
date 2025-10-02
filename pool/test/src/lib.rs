#[cfg(test)]
mod tests {
    use testlib::*;

    import!(
        name = "token-a",
        height = 0,
        tx_index = 0,
        path = "../token/contract/wit",
    );

    import!(
        name = "token-b",
        height = 0,
        tx_index = 0,
        path = "../token/contract/wit",
    );

    import!(
        name = "pool",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
    );

    interface!(name = "token-dyn", path = "../token/contract/wit",);

    #[tokio::test]
    async fn test_contract() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[
                    ("pool", &contract_bytes().await?),
                    ("token-a", &dep_contract_bytes("token").await?),
                    ("token-b", &dep_contract_bytes("token").await?),
                ])
                .build(),
        )
        .await?;

        let token_a = ContractAddress {
            name: "token-a".to_string(),
            height: 0,
            tx_index: 0,
        };

        let token_b = ContractAddress {
            name: "token-b".to_string(),
            height: 0,
            tx_index: 0,
        };

        let pool_address = ContractAddress {
            name: "pool".to_string(),
            height: 0,
            tx_index: 0,
        };

        let admin = "test_admin";
        let minter = "test_minter";
        token_a::mint(&runtime, minter, 1000.into()).await?;
        token_b::mint(&runtime, minter, 1000.into()).await?;

        token_a::transfer(&runtime, minter, admin, 100.into()).await??;
        token_b::transfer(&runtime, minter, admin, 500.into()).await??;

        let res = pool::re_init(
            &runtime,
            admin,
            token_a.clone(),
            100.into(),
            token_b.clone(),
            500.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(223.into()));

        let bal_a = pool::token_balance(&runtime, token_a.clone()).await?;
        assert_eq!(bal_a, Ok(100.into()));
        let bal_b = pool::token_balance(&runtime, token_b.clone()).await?;
        assert_eq!(bal_b, Ok(500.into()));
        let k1 = bal_a.unwrap() * bal_b.unwrap();

        let res = pool::quote_swap(&runtime, token_a.clone(), 10.into()).await?;
        assert_eq!(res, Ok(45.into()));

        let res = pool::quote_swap(&runtime, token_a.clone(), 100.into()).await?;
        assert_eq!(res, Ok(250.into()));

        let res = pool::quote_swap(&runtime, token_a.clone(), 1000.into()).await?;
        assert_eq!(res, Ok(454.into()));

        let res = pool::swap(&runtime, minter, token_a.clone(), 10.into(), 46.into()).await?;
        assert!(res.is_err()); // below minimum

        let res = pool::swap(&runtime, minter, token_a.clone(), 10.into(), 45.into()).await?;
        assert_eq!(res, Ok(45.into()));

        let bal_a = pool::token_balance(&runtime, token_a.clone()).await?;
        let bal_b = pool::token_balance(&runtime, token_b.clone()).await?;
        let k2 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k2 >= k1);

        let res = pool::quote_swap(&runtime, token_b.clone(), 45.into()).await?;
        assert_eq!(res, Ok(9.into()));
        let res = pool::swap(&runtime, minter, token_b.clone(), 45.into(), 0.into()).await?;
        assert_eq!(res, Ok(9.into()));

        let bal_a = pool::token_balance(&runtime, token_a.clone()).await?;
        let bal_b = pool::token_balance(&runtime, token_b.clone()).await?;
        let k3 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k3 >= k2);

        // use token interface to transfer shares
        let res = token_dyn::balance(&runtime, &pool_address, admin).await?;
        assert_eq!(res, Some(223.into()));
        let res = token_dyn::balance(&runtime, &pool_address, minter).await?;
        assert_eq!(res, None);

        token_dyn::transfer(&runtime, &pool_address, admin, minter, 23.into()).await??;

        let res = token_dyn::balance(&runtime, &pool_address, admin).await?;
        assert_eq!(res, Some(200.into()));
        let res = token_dyn::balance(&runtime, &pool_address, minter).await?;
        assert_eq!(res, Some(23.into()));

        Ok(())
    }
}
