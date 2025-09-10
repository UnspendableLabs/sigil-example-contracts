#[cfg(test)]
mod tests {
    use testlib::*;

    import!(
        name = "shared-account",
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
                    ("shared-account", &contract_bytes().await?),
                    ("token", &dep_contract_bytes("token").await?),
                ])
                .build(),
        )
        .await?;

        let alice = "alice";
        let bob = "bob";
        let claire = "claire";
        let dara = "dara";

        token::mint(&runtime, alice, 100.into()).await?;

        let account_id =
            shared_account::open(&runtime, alice, 50.into(), vec![bob, dara]).await??;

        let result = shared_account::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(50.into()));

        shared_account::deposit(&runtime, alice, &account_id, 25.into()).await??;

        let result = shared_account::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(75.into()));

        shared_account::withdraw(&runtime, bob, &account_id, 25.into()).await??;

        let result = shared_account::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(50.into()));

        shared_account::withdraw(&runtime, alice, &account_id, 50.into()).await??;

        let result = shared_account::balance(&runtime, &account_id).await?;
        assert_eq!(result, Some(0.into()));

        let result = shared_account::withdraw(&runtime, bob, &account_id, 1.into()).await?;
        assert_eq!(result, Err(Error::new("insufficient balance")));

        let result = shared_account::withdraw(&runtime, claire, &account_id, 1.into()).await?;
        assert_eq!(result, Err(Error::new("unauthorized")));

        let result = shared_account::tenants(&runtime, &account_id).await?;
        assert_eq!(
            result,
            Some(vec![alice.to_string(), bob.to_string(), dara.to_string()])
        );

        Ok(())
    }
}
