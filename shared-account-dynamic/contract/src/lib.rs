use stdlib::*;

contract!(name = "shared-account-dynamic");

interface!(name = "token-interface", path = "../token/contract/wit");

#[derive(Clone, Storage)]
struct Account {
    pub other_tenants: Map<String, bool>,
    pub token: ContractAddress,
    pub balance: Integer,
    pub owner: String,
}

#[derive(Clone, Default, StorageRoot)]
struct SharedAccountStorage {
    pub accounts: Map<String, Account>,
}

fn authorized(ctx: &ProcContext, token: &ContractAddress, account: &AccountWrapper) -> bool {
    (account.owner(ctx) == ctx.signer().to_string()
        || account
            .other_tenants()
            .get(ctx, ctx.signer().to_string())
            .is_some_and(|b| b))
        && token == &account.token(ctx)
}

fn insufficient_balance_error() -> Error {
    Error::new("insufficient balance")
}

fn unauthorized_error() -> Error {
    Error::new("unauthorized")
}

fn unknown_error() -> Error {
    Error::new("unknown account")
}

impl Guest for SharedAccountDynamic {
    fn init(ctx: &ProcContext) {
        SharedAccountStorage::default().init(ctx);
    }

    fn open(
        ctx: &ProcContext,
        token: ContractAddress,
        n: Integer,
        other_tenants: Vec<String>,
    ) -> Result<String, Error> {
        let balance = token_interface::balance(&token, &ctx.signer().to_string())
            .ok_or(insufficient_balance_error())?;
        if balance < n {
            return Err(insufficient_balance_error());
        }
        let account_id = crypto::generate_id();
        storage(ctx).accounts().set(
            ctx,
            account_id.clone(),
            Account {
                token: token.clone(),
                balance: n,
                owner: ctx.signer().to_string(),
                other_tenants: Map::new(
                    &other_tenants
                        .into_iter()
                        .map(|t| (t, true))
                        .collect::<Vec<_>>(),
                ),
            },
        );
        token_interface::transfer(&token, ctx.signer(), &ctx.contract_signer().to_string(), n)?;
        Ok(account_id)
    }

    fn deposit(
        ctx: &ProcContext,
        token: ContractAddress,
        account_id: String,
        n: Integer,
    ) -> Result<(), Error> {
        let balance = token_interface::balance(&token, &ctx.signer().to_string())
            .ok_or(insufficient_balance_error())?;
        if balance < n {
            return Err(insufficient_balance_error());
        }
        let account = storage(ctx)
            .accounts()
            .get(ctx, account_id)
            .ok_or(unknown_error())?;
        if !authorized(ctx, &token, &account) {
            return Err(unauthorized_error());
        }
        account.set_balance(ctx, account.balance(ctx) + n);
        token_interface::transfer(&token, ctx.signer(), &ctx.contract_signer().to_string(), n)
    }

    fn withdraw(
        ctx: &ProcContext,
        token: ContractAddress,
        account_id: String,
        n: Integer,
    ) -> Result<(), Error> {
        let account = storage(ctx)
            .accounts()
            .get(ctx, account_id)
            .ok_or(unknown_error())?;
        if !authorized(ctx, &token, &account) {
            return Err(unauthorized_error());
        }
        let balance = account.balance(ctx);
        if balance < n {
            return Err(insufficient_balance_error());
        }
        account.set_balance(ctx, balance - n);
        token_interface::transfer(&token, ctx.contract_signer(), &ctx.signer().to_string(), n)
    }

    fn balance(ctx: &ViewContext, account_id: String) -> Option<Integer> {
        storage(ctx)
            .accounts()
            .get(ctx, account_id)
            .map(|a| a.balance(ctx))
    }

    fn tenants(ctx: &ViewContext, account_id: String) -> Option<Vec<String>> {
        storage(ctx).accounts().get(ctx, account_id).map(|a| {
            [a.owner(ctx)]
                .into_iter()
                .chain(a.other_tenants().keys(ctx))
                .collect()
        })
    }
}
