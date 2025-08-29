use stdlib::*;

contract!(name = "hello-world");

impl Guest for HelloWorld {
    fn init(_ctx: &ProcContext) {}

    fn hello_world(_: &ViewContext) -> String {
        "Hello, World!".to_string()
    }
}
