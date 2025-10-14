use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("RebaLance1111111111111111111111111111111111");

// Token mint addresses (these should match your actual token mints)
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // devnet USDC
pub const USDT_MINT: Pubkey = pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"); // devnet USDT

const REBALANCE_CONFIG_SEED: &[u8] = b"rebalance-config";
const VAULT_AUTHORITY_SEED: &[u8] = b"vault-authority";
const VAULT_SEED: &[u8] = b"vault";


#[program]
pub mod rebalance {
    use super::*;

    pub fn initialize_rebalance(
        ctx: Context<InitializeRebalance>,
        params: InitializeRebalanceParams,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        // Validate admin is not the default pubkey
        require!(
            ctx.accounts.admin.key() != Pubkey::default(),
            RebalanceError::InvalidAdmin
        );

        let InitializeRebalanceBumps {
            config: config_bump,
            vault_authority: vault_authority_bump,
            ..
        } = ctx.bumps;
        config.bump = config_bump;
        config.vault_authority_bump = vault_authority_bump;
        config.admin = ctx.accounts.admin.key();
        config.guardian = if params.guardian == Pubkey::default() {
            ctx.accounts.admin.key()
        } else {
            params.guardian
        };
        config.paused = false;
        config.last_rebalance = 0;
        config.total_rebalances = 0;

        emit!(RebalanceInitializedEvent {
            admin: config.admin,
            guardian: config.guardian,
        });

        Ok(())
    }

    pub fn rebalance_once(ctx: Context<ExecuteRebalance>) -> Result<()> {
        let config = &ctx.accounts.config;

        // Authorization check: Only admin or guardian can trigger rebalancing
        require!(
            ctx.accounts.authority.key() == config.admin || ctx.accounts.authority.key() == config.guardian,
            RebalanceError::UnauthorizedRebalance
        );

        require!(!config.paused, RebalanceError::RebalancingPaused);

        let current_time = Clock::get()?.unix_timestamp;

        // SECURE: Read actual balances from validated token vaults
        let usdc_balance: u64 = ctx.accounts.usdc_vault.amount;
        let usdt_balance: u64 = ctx.accounts.usdt_vault.amount;
        let total_balance = usdc_balance
            .checked_add(usdt_balance)
            .ok_or(RebalanceError::MathOverflow)?;

        require!(total_balance > 0, RebalanceError::NoAssetsToRebalance);

        // For MVP, implement simple 50/50 rebalancing
        let target_usdc = total_balance
            .checked_div(2)
            .ok_or(RebalanceError::MathOverflow)?;
        let target_usdt = total_balance
            .checked_sub(target_usdc)
            .ok_or(RebalanceError::MathOverflow)?;

        // Check if rebalance is needed (5% threshold)
        let usdc_diff = if usdc_balance > target_usdc {
            usdc_balance.checked_sub(target_usdc)
        } else {
            target_usdc.checked_sub(usdc_balance)
        }
        .ok_or(RebalanceError::MathOverflow)?;

        let drift_threshold_amount = total_balance
            .checked_mul(500) // 5%
            .ok_or(RebalanceError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(RebalanceError::MathOverflow)?;

        if usdc_diff <= drift_threshold_amount {
            return err!(RebalanceError::NoRebalanceNeeded);
        }

        // Determine swap direction
        let (from_usdc_to_usdt, swap_amount) = if usdc_balance > target_usdc {
            (
                true,
                usdc_balance
                    .checked_sub(target_usdc)
                    .ok_or(RebalanceError::MathOverflow)?,
            )
        } else {
            (
                false,
                usdt_balance
                    .checked_sub(target_usdt)
                    .ok_or(RebalanceError::MathOverflow)?,
            )
        };

        require!(swap_amount > 0, RebalanceError::ZeroSwapAmount);

        // For MVP: Log rebalance details instead of executing actual swaps
        // In production: implement DEX CPI here using vault_authority as signer
        msg!(
            "Rebalance needed: {} {} -> {}",
            if from_usdc_to_usdt { "USDC" } else { "USDT" },
            swap_amount,
            if from_usdc_to_usdt { "USDT" } else { "USDC" }
        );
        msg!(
            "Target composition: {} USDC, {} USDT",
            target_usdc,
            target_usdt
        );

        // TODO: Implement secure swap using program-controlled vaults
        // Example secure transfer pattern:
        // let vault_authority_bump = [config.vault_authority_bump];
        // let signer_seeds: &[&[&[u8]]] = &[&[VAULT_AUTHORITY_SEED, &vault_authority_bump]];
        // let transfer_ctx = CpiContext::new_with_signer(
        //     ctx.accounts.token_program.to_account_info(),
        //     Transfer {
        //         from: source_vault.to_account_info(),
        //         to: dest_vault.to_account_info(),
        //         authority: ctx.accounts.vault_authority.to_account_info(),
        //     },
        //     signer_seeds,
        // );
        // transfer(transfer_ctx, swap_amount)?;

        // Update rebalance config
        let config = &mut ctx.accounts.config;
        config.last_rebalance = current_time;
        config.total_rebalances = config
            .total_rebalances
            .checked_add(1)
            .ok_or(RebalanceError::MathOverflow)?;

        emit!(RebalancedEvent {
            usdc_before: usdc_balance,
            usdt_before: usdt_balance,
            usdc_target: target_usdc,
            usdt_target: target_usdt,
            swap_amount,
            from_usdc_to_usdt,
            timestamp: current_time,
        });

        Ok(())
    }

    pub fn pause_rebalancing(ctx: Context<ToggleRebalancing>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.paused = true;

        emit!(RebalancePausedEvent {
            guardian: ctx.accounts.guardian.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn unpause_rebalancing(ctx: Context<ToggleRebalancing>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.paused = false;

        emit!(RebalanceUnpausedEvent {
            guardian: ctx.accounts.guardian.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeRebalanceParams {
    pub guardian: Pubkey,
}

#[account]
pub struct RebalanceConfig {
    pub bump: u8,
    pub vault_authority_bump: u8,
    pub admin: Pubkey,
    pub guardian: Pubkey,
    pub paused: bool,
    pub last_rebalance: i64,
    pub total_rebalances: u64,
}

impl RebalanceConfig {
    pub const SPACE: usize = 8 + 1 + 1 + (2 * 32) + 1 + 8 + 8; // 91 bytes
}

#[derive(Accounts)]
pub struct InitializeRebalance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = RebalanceConfig::SPACE,
        seeds = [REBALANCE_CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, RebalanceConfig>,
    /// CHECK: PDA for vault authority
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump)]
    pub vault_authority: AccountInfo<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [VAULT_SEED, USDC_MINT.as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = vault_authority
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = payer,
        seeds = [VAULT_SEED, USDT_MINT.as_ref()],
        bump,
        token::mint = usdt_mint,
        token::authority = vault_authority
    )]
    pub usdt_vault: Account<'info, TokenAccount>,
    pub usdc_mint: Account<'info, anchor_spl::token::Mint>,
    pub usdt_mint: Account<'info, anchor_spl::token::Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteRebalance<'info> {
    #[account(
        mut,
        seeds = [REBALANCE_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, RebalanceConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: PDA for vault authority
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump = config.vault_authority_bump)]
    pub vault_authority: AccountInfo<'info>,

    // Program-controlled token vaults (PDAs)
    #[account(
        mut,
        seeds = [VAULT_SEED, USDC_MINT.as_ref()],
        bump,
        constraint = usdc_vault.mint == USDC_MINT,
    )]
    pub usdc_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [VAULT_SEED, USDT_MINT.as_ref()],
        bump,
        constraint = usdt_vault.mint == USDT_MINT,
    )]
    pub usdt_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ToggleRebalancing<'info> {
    #[account(mut)]
    pub guardian: Signer<'info>,
    #[account(
        mut,
        has_one = guardian,
        seeds = [REBALANCE_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, RebalanceConfig>,
}

#[event]
pub struct RebalanceInitializedEvent {
    pub admin: Pubkey,
    pub guardian: Pubkey,
}

#[event]
pub struct RebalancedEvent {
    pub usdc_before: u64,
    pub usdt_before: u64,
    pub usdc_target: u64,
    pub usdt_target: u64,
    pub swap_amount: u64,
    pub from_usdc_to_usdt: bool,
    pub timestamp: i64,
}

#[event]
pub struct RebalancePausedEvent {
    pub guardian: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RebalanceUnpausedEvent {
    pub guardian: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum RebalanceError {
    #[msg("Rebalancing is currently paused")]
    RebalancingPaused,
    #[msg("No assets available to rebalance")]
    NoAssetsToRebalance,
    #[msg("Mathematical overflow occurred")]
    MathOverflow,
    #[msg("Current composition is within drift threshold - no rebalance needed")]
    NoRebalanceNeeded,
    #[msg("Swap amount must be greater than zero")]
    ZeroSwapAmount,
    #[msg("Only admin or guardian can trigger rebalancing")]
    UnauthorizedRebalance,
    #[msg("Admin cannot be the default pubkey")]
    InvalidAdmin,
}
