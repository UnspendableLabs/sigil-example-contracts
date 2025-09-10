#[cfg(test)]
mod tests {
    use testlib::*;

    import!(
        name = "hello-world",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
    );

    #[tokio::test]
    async fn test_contract() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[("hello-world", &contract_bytes().await?)])
                .build(),
        )
        .await?;

        let result = hello_world::hello_world(&runtime).await?;
        assert_eq!(result, "Hello, World!");

        Ok(())
    }
}
