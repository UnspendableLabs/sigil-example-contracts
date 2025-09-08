use stdlib::*;

contract!(name = "shared-account-dynamic");

impl Guest for SharedAccountDynamic {
    fn init(_ctx: &ProcContext) {}

    fn hello_world(_ctx: &ViewContext) -> String {
        "Hello, World!".to_string()
    }
}
