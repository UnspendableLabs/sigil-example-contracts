use stdlib::*;

contract!(name = "token");

impl Guest for Token {
    fn init(_ctx: &ProcContext) {}

    fn hello_world(_ctx: &ViewContext) -> String {
        "Hello, World!".to_string()
    }
}
