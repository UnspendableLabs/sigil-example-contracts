use stdlib::*;

contract!(name = "token");

#[derive(Clone, Default, StorageRoot)]
struct TokenStorage {
    pub ledger: Map<String, Integer>,
}

impl Guest for Token {
    fn init(ctx: &ProcContext) {
        TokenStorage::default().init(ctx);
    }

    fn mint(ctx: &ProcContext, n: Integer) {
        let to = ctx.signer().to_string();
        let ledger = storage(ctx).ledger();

        let balance = ledger.get(ctx, &to).unwrap_or_default();
        ledger.set(ctx, to, balance + n);
    }

    fn transfer(ctx: &ProcContext, to: String, n: Integer) -> Result<(), Error> {
        let from = ctx.signer().to_string();
        let ledger = storage(ctx).ledger();

        let from_balance = ledger.get(ctx, &from).unwrap_or_default();
        let to_balance = ledger.get(ctx, &to).unwrap_or_default();

        if from_balance < n {
            return Err(Error::new("insufficient funds"));
        }

        ledger.set(ctx, from, from_balance - n);
        ledger.set(ctx, to, to_balance + n);
        Ok(())
    }

    fn balance(ctx: &ViewContext, acc: String) -> Option<Integer> {
        storage(ctx).ledger().get(ctx, acc)
    }
}
