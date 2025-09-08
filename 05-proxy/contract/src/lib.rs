use stdlib::*;

contract!(name = "proxy");

impl Guest for Proxy {
    fn init(_ctx: &ProcContext) {}

    fn hello_world(_ctx: &ViewContext) -> String {
        "Hello, World!".to_string()
    }
}
