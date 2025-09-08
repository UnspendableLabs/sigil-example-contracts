#[cfg(test)]
mod tests {
    use testlib::*;

    import!(
        name = "proxy",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
        test = true,
    );

    #[tokio::test]
    async fn test_contract() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[("proxy", &contract_bytes().await?)])
                .build(),
        )
        .await?;

        let result = proxy::hello_world(&runtime).await?;
        assert_eq!(result, "Hello, World!");

        Ok(())
    }
}
