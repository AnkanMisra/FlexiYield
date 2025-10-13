use anchor_lang::prelude::*;

declare_id!("BaskEt11111111111111111111111111111111111111");

#[program]
pub mod basket {
    use super::*;

    pub fn initialize_basket(
        _ctx: Context<InitializeBasket>,
        _params: InitializeBasketParams,
    ) -> Result<()> {
        Ok(())
    }

    pub fn deposit_usdc(_ctx: Context<DepositUsdc>, _amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn redeem_flex(_ctx: Context<RedeemFlex>, _amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn update_config(
        _ctx: Context<UpdateConfig>,
        _params: UpdateConfigParams,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeBasketParams {}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UpdateConfigParams {}

#[derive(Accounts)]
pub struct InitializeBasket<'info> {
    // TODO: wire up basket config PDA, authority, and token vaults
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositUsdc<'info> {
    // TODO: define vaults and token accounts
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RedeemFlex<'info> {
    // TODO: define vaults and token accounts
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    // TODO: gate updates behind the configured admin authority
    pub admin: Signer<'info>,
}

#[event]
pub struct DepositEvent {
    pub amount: u64,
}

#[event]
pub struct RedeemEvent {
    pub amount: u64,
}
