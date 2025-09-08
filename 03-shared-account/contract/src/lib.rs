use stdlib::*;

contract!(name = "shared-account");

impl Guest for SharedAccount {
    fn init(_ctx: &ProcContext) {}

    fn hello_world(_ctx: &ViewContext) -> String {
        "Hello, World!".to_string()
    }
}
