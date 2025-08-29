#[cfg(test)]
mod tests {
    use testlib::*;

    const WASM_BYTES: &[u8] = include_bytes!(env!("CONTRACT_WASM_PATH"));

    import!(
        name = "hello-world",
        height = 0,
        tx_index = 0,
        path = "contract/wit",
        test = true,
    );

    #[tokio::test]
    async fn test_contract() -> Result<()> {
        let runtime = Runtime::new(
            RuntimeConfig::builder()
                .contracts(&[("hello-world", WASM_BYTES)])
                .build(),
        )
        .await?;

        let result = hello_world::hello_world(&runtime).await;
        assert_eq!(result, "Hello, World!");

        Ok(())
    }
}
