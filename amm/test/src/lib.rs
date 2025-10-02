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
        name = "amm",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
    );

    #[tokio::test]
    async fn test_contract() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[
                    ("amm", &contract_bytes().await?),
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

        let admin = "test_admin";
        let minter = "test_minter";
        token_a::mint(&runtime, minter, 1000.into()).await?;
        token_b::mint(&runtime, minter, 1000.into()).await?;

        token_a::transfer(&runtime, minter, admin, 100.into()).await??;
        token_b::transfer(&runtime, minter, admin, 500.into()).await??;

        let pair = amm::TokenPair {
            a: token_a.clone(),
            b: token_b.clone(),
        };
        let res = amm::create(
            &runtime,
            admin,
            pair.clone(),
            100.into(),
            500.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(223.into()));

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        assert_eq!(bal_a, Ok(100.into()));
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        assert_eq!(bal_b, Ok(500.into()));
        let k1 = bal_a.unwrap() * bal_b.unwrap();

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 10.into()).await?;
        assert_eq!(res, Ok(45.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 100.into()).await?;
        assert_eq!(res, Ok(250.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 1000.into()).await?;
        assert_eq!(res, Ok(454.into()));

        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_a.clone(),
            10.into(),
            46.into(),
        )
        .await?;
        assert!(res.is_err()); // below minimum

        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_a.clone(),
            10.into(),
            45.into(),
        )
        .await?;
        assert_eq!(res, Ok(45.into()));

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k2 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k2 >= k1);

        let res = amm::quote_swap(&runtime, pair.clone(), token_b.clone(), 45.into()).await?;
        assert_eq!(res, Ok(9.into()));
        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_b.clone(),
            45.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(9.into()));

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k3 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k3 >= k2);

        Ok(())
    }
}
