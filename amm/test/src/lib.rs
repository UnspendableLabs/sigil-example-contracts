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
        name = "token-c",
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
    async fn test_amm_swaps() -> Result<()> {
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

    #[tokio::test]
    async fn test_amm_swap_fee() -> Result<()> {
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
        amm::create(
            &runtime,
            admin,
            pair.clone(),
            100.into(),
            500.into(),
            30.into(),
        )
        .await??;

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k1 = bal_a.unwrap() * bal_b.unwrap();

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 10.into()).await?;
        assert_eq!(res, Ok(41.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 100.into()).await?;
        assert_eq!(res, Ok(248.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 1000.into()).await?;
        assert_eq!(res, Ok(454.into())); // fee dominated by rounding effect

        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_a.clone(),
            10.into(),
            40.into(),
        )
        .await?;
        assert_eq!(res, Ok(41.into()));

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

    #[tokio::test]
    async fn test_amm_swap_low_slippage() -> Result<()> {
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
        token_a::mint(&runtime, minter, 110000.into()).await?;
        token_b::mint(&runtime, minter, 510000.into()).await?;

        token_a::transfer(&runtime, minter, admin, 100000.into()).await??;
        token_b::transfer(&runtime, minter, admin, 500000.into()).await??;

        let pair = amm::TokenPair {
            a: token_a.clone(),
            b: token_b.clone(),
        };
        amm::create(
            &runtime,
            admin,
            pair.clone(),
            100000.into(),
            500000.into(),
            30.into(),
        )
        .await??;

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k1 = bal_a.unwrap() * bal_b.unwrap();

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 10.into()).await?;
        assert_eq!(res, Ok(44.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 100.into()).await?;
        assert_eq!(res, Ok(494.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 1000.into()).await?;
        assert_eq!(res, Ok(4935.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 10000.into()).await?;
        assert_eq!(res, Ok(45330.into()));

        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_a.clone(),
            10000.into(),
            45000.into(),
        )
        .await?;
        assert_eq!(res, Ok(45330.into()));

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k2 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k2 >= k1 + (30 * 450000).into()); // grows with fee amount

        let res = amm::quote_swap(&runtime, pair.clone(), token_b.clone(), 45.into()).await?;
        assert_eq!(res, Ok(10.into()));
        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_b.clone(),
            45.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(10.into()));

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k3 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k3 >= k2);

        Ok(())
    }

    #[tokio::test]
    async fn test_amm_deposit_withdraw() -> Result<()> {
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
        let holder = "test_holder";
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

        token_a::transfer(&runtime, minter, holder, 200.into()).await??;
        token_b::transfer(&runtime, minter, holder, 200.into()).await??;

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        assert_eq!(bal_a, Ok(100.into()));
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        assert_eq!(bal_b, Ok(500.into()));

        let res = amm::quote_withdraw(&runtime, pair.clone(), 10.into()).await?;
        assert_eq!(
            res,
            Ok(amm::WithdrawResult {
                amount_a: 4.into(),
                amount_b: 22.into(),
            })
        );

        let res = amm::quote_deposit(&runtime, pair.clone(), 10.into(), 100.into()).await?;
        assert_eq!(
            res,
            Ok(amm::DepositResult {
                lp_shares: 22.into(),
                deposit_a: 10.into(),
                deposit_b: 50.into(),
            })
        );

        let res = amm::deposit(&runtime, holder, pair.clone(), 50.into(), 100.into()).await?;
        assert_eq!(
            res,
            Ok(amm::DepositResult {
                lp_shares: 44.into(),
                deposit_a: 20.into(),
                deposit_b: 99.into(),
            })
        );

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        assert_eq!(bal_a, Ok(120.into()));
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        assert_eq!(bal_b, Ok(599.into()));

        let bal = amm::balance(&runtime, pair.clone(), admin).await?;
        assert_eq!(bal, Some(223.into()));
        let bal = amm::balance(&runtime, pair.clone(), holder).await?;
        assert_eq!(bal, Some(44.into()));

        let res = amm::quote_withdraw(&runtime, pair.clone(), 10.into()).await?;
        assert_eq!(
            res,
            Ok(amm::WithdrawResult {
                amount_a: 4.into(),
                amount_b: 22.into(),
            })
        );

        let res = amm::withdraw(&runtime, holder, pair.clone(), 44.into()).await?;
        assert_eq!(
            res,
            Ok(amm::WithdrawResult {
                amount_a: 19.into(),
                amount_b: 98.into(),
            })
        );

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        assert_eq!(bal_a, Ok(101.into()));
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        assert_eq!(bal_b, Ok(501.into()));

        let bal = amm::balance(&runtime, pair.clone(), admin).await?;
        assert_eq!(bal, Some(223.into()));
        let bal = amm::balance(&runtime, pair.clone(), holder).await?;
        assert_eq!(bal, Some(0.into()));

        Ok(())
    }

    #[tokio::test]
    async fn test_amm_limits() -> Result<()> {
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

        let max_int =
            "115_792_089_237_316_195_423_570_985_008_687_907_853_269_984_665_640_564_039_457";
        let large_value: Integer = "340_282_366_920_938_463_463_374_606_431".into(); // sqrt(MAX_INT) - 1000
        let oversized_value = large_value + 1.into();

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
        token_a::mint(&runtime, minter, max_int.into()).await?;
        token_b::mint(&runtime, minter, max_int.into()).await?;

        token_a::transfer(&runtime, minter, admin, 1000.into()).await??;
        token_b::transfer(&runtime, minter, admin, 1000.into()).await??;

        let pair = amm::TokenPair {
            a: token_a.clone(),
            b: token_b.clone(),
        };
        let res = amm::create(
            &runtime,
            admin,
            pair.clone(),
            1000.into(),
            1000.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(1000.into()));

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k1 = bal_a.unwrap() * bal_b.unwrap();

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), large_value).await?;
        assert_eq!(res, Ok(999.into()));
        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), oversized_value).await?;
        assert!(res.is_err());

        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_a.clone(),
            large_value,
            900.into(),
        )
        .await?;
        assert_eq!(res, Ok(999.into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_a.clone(), 1.into()).await?;
        assert!(res.is_err());
        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_a.clone(),
            1.into(),
            0.into(),
        )
        .await?;
        assert!(res.is_err());

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k2 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k2 >= k1);

        let res = amm::quote_swap(&runtime, pair.clone(), token_b.clone(), large_value).await?;
        assert_eq!(res, Ok("340_282_366_920_938_463_463_374_607_429".into()));
        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_b.clone(),
            large_value,
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok("340_282_366_920_938_463_463_374_607_429".into()));

        let res = amm::quote_swap(&runtime, pair.clone(), token_b.clone(), 1000.into()).await?;
        assert!(res.is_err());
        let res = amm::swap(
            &runtime,
            minter,
            pair.clone(),
            token_b.clone(),
            1000.into(),
            0.into(),
        )
        .await?;
        assert!(res.is_err());

        let bal_a = amm::token_balance(&runtime, pair.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair.clone(), token_b.clone()).await?;
        let k3 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k3 >= k2);

        Ok(())
    }

    #[tokio::test]
    async fn test_amm_pools() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[
                    ("amm", &contract_bytes().await?),
                    ("token-a", &dep_contract_bytes("token").await?),
                    ("token-b", &dep_contract_bytes("token").await?),
                    ("token-c", &dep_contract_bytes("token").await?),
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
        let token_c = ContractAddress {
            name: "token-c".to_string(),
            height: 0,
            tx_index: 0,
        };

        let admin = "test_admin";
        let minter = "test_minter";
        token_a::mint(&runtime, minter, 1000.into()).await?;
        token_b::mint(&runtime, minter, 1000.into()).await?;
        token_c::mint(&runtime, minter, 1000.into()).await?;

        token_a::transfer(&runtime, minter, admin, 100.into()).await??;
        token_b::transfer(&runtime, minter, admin, 600.into()).await??;
        token_c::transfer(&runtime, minter, admin, 200.into()).await??;

        let bad_pair = amm::TokenPair {
            // wrong order
            a: token_b.clone(),
            b: token_a.clone(),
        };
        let res = amm::create(
            &runtime,
            admin,
            bad_pair.clone(),
            100.into(),
            500.into(),
            0.into(),
        )
        .await?;
        assert!(res.is_err());

        let pair1 = amm::TokenPair {
            a: token_a.clone(),
            b: token_b.clone(),
        };
        let pair2 = amm::TokenPair {
            a: token_b.clone(),
            b: token_c.clone(),
        };
        let res = amm::create(
            &runtime,
            admin,
            pair1.clone(),
            100.into(),
            500.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(223.into()));

        let res = amm::create(
            &runtime,
            admin,
            pair1.clone(),
            100.into(),
            500.into(),
            0.into(),
        )
        .await?;
        assert!(res.is_err()); // can't create pool twice

        let res = amm::create(
            &runtime,
            admin,
            pair2.clone(),
            100.into(),
            200.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(141.into()));

        let bal_a = amm::token_balance(&runtime, pair1.clone(), token_a.clone()).await?;
        assert_eq!(bal_a, Ok(100.into()));
        let bal_b = amm::token_balance(&runtime, pair1.clone(), token_b.clone()).await?;
        assert_eq!(bal_b, Ok(500.into()));
        let k1_1 = bal_a.unwrap() * bal_b.unwrap();

        let bal_b = amm::token_balance(&runtime, pair2.clone(), token_b.clone()).await?;
        assert_eq!(bal_b, Ok(100.into()));
        let bal_c = amm::token_balance(&runtime, pair2.clone(), token_c.clone()).await?;
        assert_eq!(bal_c, Ok(200.into()));
        let k2_1 = bal_b.unwrap() * bal_c.unwrap();

        let res = amm::quote_swap(&runtime, pair1.clone(), token_a.clone(), 10.into()).await?;
        assert_eq!(res, Ok(45.into()));

        let res = amm::quote_swap(&runtime, pair1.clone(), token_a.clone(), 100.into()).await?;
        assert_eq!(res, Ok(250.into()));

        let res = amm::quote_swap(&runtime, pair1.clone(), token_a.clone(), 1000.into()).await?;
        assert_eq!(res, Ok(454.into()));

        let res = amm::swap(
            &runtime,
            minter,
            pair1.clone(),
            token_a.clone(),
            10.into(),
            45.into(),
        )
        .await?;
        assert_eq!(res, Ok(45.into()));

        let bal_a = amm::token_balance(&runtime, pair1.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair1.clone(), token_b.clone()).await?;
        let k1_2 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k1_2 >= k1_1);

        let bal_b = amm::token_balance(&runtime, pair2.clone(), token_b.clone()).await?;
        let bal_c = amm::token_balance(&runtime, pair2.clone(), token_c.clone()).await?;
        let k2_2 = bal_b.unwrap() * bal_c.unwrap();
        assert!(k2_2 == k2_1); // unchanged

        let res = amm::quote_swap(&runtime, pair1.clone(), token_b.clone(), 45.into()).await?;
        assert_eq!(res, Ok(9.into()));
        let res = amm::swap(
            &runtime,
            minter,
            pair1.clone(),
            token_b.clone(),
            45.into(),
            0.into(),
        )
        .await?;
        assert_eq!(res, Ok(9.into()));

        let bal_a = amm::token_balance(&runtime, pair1.clone(), token_a.clone()).await?;
        let bal_b = amm::token_balance(&runtime, pair1.clone(), token_b.clone()).await?;
        let k1_3 = bal_a.unwrap() * bal_b.unwrap();
        assert!(k1_3 >= k1_2);

        let bal_b = amm::token_balance(&runtime, pair2.clone(), token_b.clone()).await?;
        let bal_c = amm::token_balance(&runtime, pair2.clone(), token_c.clone()).await?;
        let k2_3 = bal_b.unwrap() * bal_c.unwrap();
        assert!(k2_3 == k2_1); // unchanged

        Ok(())
    }
}
