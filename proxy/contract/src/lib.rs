use stdlib::*;

contract!(name = "proxy");

#[derive(Clone, Default, StorageRoot)]
struct ProxyStorage {
    contract_address: Option<ContractAddress>,
}

impl Guest for Proxy {
    fn init(ctx: &ProcContext) {
        ProxyStorage::default().init(ctx)
    }

    fn fallback(ctx: &FallContext, expr: String) -> String {
        let _ctx = &ctx.view_context();
        let contract_address = storage(_ctx).contract_address(_ctx).unwrap();
        foreign::call(ctx.signer(), &contract_address, &expr)
    }

    fn get_contract_address(ctx: &ViewContext) -> Option<ContractAddress> {
        storage(ctx).contract_address(ctx)
    }

    fn set_contract_address(ctx: &ProcContext, contract_address: ContractAddress) {
        storage(ctx).set_contract_address(ctx, Some(contract_address));
    }
}
