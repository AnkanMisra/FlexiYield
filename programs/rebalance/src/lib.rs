use anchor_lang::prelude::*;

declare_id!("RebaLanCe11111111111111111111111111111111");

#[program]
pub mod rebalance {
    use super::*;

    pub fn rebalance_once(_ctx: Context<ExecuteRebalance>) -> Result<()> {
        Ok(())
    }

    pub fn pause_rebalancing(_ctx: Context<ToggleRebalancing>) -> Result<()> {
        Ok(())
    }

    pub fn unpause_rebalancing(_ctx: Context<ToggleRebalancing>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ExecuteRebalance<'info> {
    // TODO: wire up vault PDAs, strategy accounts, and swap interface
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ToggleRebalancing<'info> {
    // TODO: restrict to guardian authority
    pub guardian: Signer<'info>,
}

#[event]
pub struct RebalancedEvent {
    pub timestamp: i64,
}
