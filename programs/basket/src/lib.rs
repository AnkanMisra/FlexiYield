use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, Burn, MintTo, SetAuthority, Token, Transfer,
};
use anchor_spl::token::{Mint as TokenMint, TokenAccount as SplTokenAccount};

pub const BASKET_DECIMALS: u8 = 6;
const DECIMAL_FACTOR: u128 = 1_000_000;
const BPS_DENOMINATOR: u16 = 10_000;
const CONFIG_SEED: &[u8] = b"basket-config";
const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority";
const VAULT_SEED: &[u8] = b"vault";

declare_id!("BaskEt11111111111111111111111111111111111111");

#[program]
pub mod basket {
    use super::*;

    pub fn initialize_basket(
        ctx: Context<InitializeBasket>,
        params: InitializeBasketParams,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        require!(
            ctx.accounts.usdc_mint.decimals == BASKET_DECIMALS
                && ctx.accounts.usdt_mint.decimals == BASKET_DECIMALS
                && ctx.accounts.flex_mint.decimals == BASKET_DECIMALS,
            BasketError::InvalidMintDecimals
        );

        match ctx.accounts.flex_mint.mint_authority {
            COption::Some(authority) => {
                require_keys_eq!(
                    authority,
                    ctx.accounts.admin.key(),
                    BasketError::InvalidMintAuthority
                );
            }
            COption::None => return err!(BasketError::InvalidMintAuthority),
        }

        let token_program = ctx.accounts.token_program.to_account_info();
        let admin_info = ctx.accounts.admin.to_account_info();
        let flex_mint_info = ctx.accounts.flex_mint.to_account_info();

        token::set_authority(
            CpiContext::new(
                token_program.clone(),
                SetAuthority {
                    account_or_mint: flex_mint_info.clone(),
                    current_authority: admin_info.clone(),
                },
            ),
            AuthorityType::MintTokens,
            Some(ctx.accounts.mint_authority.key()),
        )?;

        token::set_authority(
            CpiContext::new(
                token_program,
                SetAuthority {
                    account_or_mint: flex_mint_info,
                    current_authority: admin_info,
                },
            ),
            AuthorityType::FreezeAccount,
            Some(ctx.accounts.mint_authority.key()),
        )?;

        let guardian = if params.guardian == Pubkey::default() {
            ctx.accounts.admin.key()
        } else {
            params.guardian
        };

        require!(
            params.max_deposit_amount > 0 && params.max_daily_deposit > 0,
            BasketError::InvalidLimits
        );

        let InitializeBasketBumps {
            config: config_bump,
            mint_authority: mint_authority_bump,
            usdc_vault: usdc_vault_bump,
            usdt_vault: usdt_vault_bump,
        } = ctx.bumps;

        config.bump = config_bump;
        config.mint_authority_bump = mint_authority_bump;
        config.usdc_vault_bump = usdc_vault_bump;
        config.usdt_vault_bump = usdt_vault_bump;
        config.admin = ctx.accounts.admin.key();
        config.guardian = guardian;
        config.emergency_admin = params.emergency_admin;
        config.flex_mint = ctx.accounts.flex_mint.key();
        config.usdc_mint = ctx.accounts.usdc_mint.key();
        config.usdt_mint = ctx.accounts.usdt_mint.key();
        config.usdc_vault = ctx.accounts.usdc_vault.key();
        config.usdt_vault = ctx.accounts.usdt_vault.key();
        config.nav = DECIMAL_FACTOR as u64;
        config.flex_supply_snapshot = 0;
        config.last_total_assets = 0;
        config.paused = false;
        config.max_deposit_amount = params.max_deposit_amount;
        config.max_daily_deposit = params.max_daily_deposit;
        config.daily_deposit_volume = 0;
        config.last_deposit_day = 0;

        emit!(ConfigInitializedEvent {
            admin: config.admin,
            guardian: config.guardian,
            emergency_admin: config.emergency_admin,
            flex_mint: config.flex_mint,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn deposit_usdc(ctx: Context<DepositUsdc>, params: DepositUsdcParams) -> Result<()> {
        require!(params.amount > 0, BasketError::AmountMustBePositive);
        require!(
            ctx.accounts.user_usdc.amount >= params.amount,
            BasketError::InsufficientUserFunds
        );

        let config = &mut ctx.accounts.config;
        config.can_deposit(params.amount)?;

        let current_time = Clock::get()?.unix_timestamp;
        config.check_daily_limit(params.amount, current_time)?;

        let flex_supply_before = ctx.accounts.flex_mint.supply as u128;
        let total_assets_before = total_assets(&ctx.accounts.usdc_vault, &ctx.accounts.usdt_vault)?;

        let flex_to_mint = if flex_supply_before == 0 {
            params.amount
        } else {
            require!(total_assets_before > 0, BasketError::ZeroTotalAssets);

            // SECURITY: Additional per-transaction limit to prevent abuse
            const MAX_SINGLE_DEPOSIT: u64 = 10_000_000; // 10 USDC max per transaction
            require!(
                params.amount <= MAX_SINGLE_DEPOSIT,
                BasketError::SingleDepositTooLarge
            );

            // SECURITY: Enhanced slippage protection with price impact checks
            let current_price = if flex_supply_before > 0 {
                total_assets_before
                    .checked_mul(DECIMAL_FACTOR)
                    .ok_or(BasketError::MathOverflow)?
                    .checked_div(flex_supply_before)
                    .ok_or(BasketError::ZeroNav)?
            } else {
                DECIMAL_FACTOR as u128
            };

            let minted_u128 = (params.amount as u128)
                .checked_mul(flex_supply_before)
                .ok_or(BasketError::MathOverflow)?
                .checked_div(total_assets_before)
                .ok_or(BasketError::ZeroNav)?;
            require!(minted_u128 > 0, BasketError::AmountTooSmallForShare);
            require!(minted_u128 <= u64::MAX as u128, BasketError::MathOverflow);

            let minted = minted_u128 as u64;

            // Basic slippage protection
            require!(minted >= params.min_flex_out, BasketError::SlippageExceeded);

            // SECURITY: Maximum price impact protection (5%)
            const MAX_PRICE_IMPACT_BPS: u16 = 500; // 5%
            let max_acceptable_price = current_price
                .checked_mul((BPS_DENOMINATOR + MAX_PRICE_IMPACT_BPS) as u128)
                .ok_or(BasketError::MathOverflow)?
                .checked_div(BPS_DENOMINATOR as u128)
                .ok_or(BasketError::MathOverflow)?;

            // Simulate post-deposit price to check for excessive impact
            let new_total_assets = total_assets_before
                .checked_add(params.amount as u128)
                .ok_or(BasketError::MathOverflow)?;
            let new_supply = flex_supply_before
                .checked_add(minted_u128)
                .ok_or(BasketError::MathOverflow)?;

            let new_price = new_total_assets
                .checked_mul(DECIMAL_FACTOR)
                .ok_or(BasketError::MathOverflow)?
                .checked_div(new_supply)
                .ok_or(BasketError::ZeroNav)?;

            require!(
                new_price <= max_acceptable_price,
                BasketError::ExcessivePriceImpact
            );

            minted
        };

        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc.to_account_info(),
                to: ctx.accounts.usdc_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, params.amount)?;

        let mint_authority_bump = [config.mint_authority_bump];
        let signer_seeds: &[&[&[u8]]] = &[&[MINT_AUTHORITY_SEED, &mint_authority_bump]];
        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.flex_mint.to_account_info(),
                to: ctx.accounts.user_flex.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(mint_ctx, flex_to_mint)?;

        let total_assets_after = total_assets_before
            .checked_add(params.amount as u128)
            .ok_or(BasketError::MathOverflow)?;
        let flex_supply_after = flex_supply_before
            .checked_add(flex_to_mint as u128)
            .ok_or(BasketError::MathOverflow)?;
        config.update_nav(total_assets_after, flex_supply_after)?;

        emit!(DepositEvent {
            depositor: ctx.accounts.user.key(),
            usdc_amount: params.amount,
            flex_minted: flex_to_mint,
            nav: config.nav,
            timestamp: current_time,
        });

        Ok(())
    }

    pub fn redeem_flex(ctx: Context<RedeemFlex>, params: RedeemFlexParams) -> Result<()> {
        require!(params.amount > 0, BasketError::AmountMustBePositive);
        require!(
            ctx.accounts.user_flex.amount >= params.amount,
            BasketError::InsufficientUserFunds
        );

        let config = &ctx.accounts.config;
        require!(!config.is_paused(), BasketError::ContractPaused);

        let current_time = Clock::get()?.unix_timestamp;
        let flex_supply_before = ctx.accounts.flex_mint.supply as u128;
        require!(flex_supply_before > 0, BasketError::ZeroSupply);

        let total_assets_before = total_assets(&ctx.accounts.usdc_vault, &ctx.accounts.usdt_vault)?;
        require!(total_assets_before > 0, BasketError::ZeroTotalAssets);

        let usdc_to_return = (params.amount as u128)
            .checked_mul(total_assets_before)
            .ok_or(BasketError::MathOverflow)?
            .checked_div(flex_supply_before)
            .ok_or(BasketError::ZeroNav)?
            .try_into()
            .map_err(|_| BasketError::MathOverflow)?;

        require!(usdc_to_return > 0, BasketError::AmountTooSmallForShare);
        require!(
            usdc_to_return >= params.min_usdc_out,
            BasketError::SlippageExceeded
        );
        require!(
            ctx.accounts.usdc_vault.amount >= usdc_to_return,
            BasketError::InsufficientVaultLiquidity
        );

        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.flex_mint.to_account_info(),
                from: ctx.accounts.user_flex.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::burn(burn_ctx, params.amount)?;

        let total_assets_after = total_assets_before
            .checked_sub(usdc_to_return as u128)
            .ok_or(BasketError::MathOverflow)?;
        let flex_supply_after = flex_supply_before
            .checked_sub(params.amount as u128)
            .ok_or(BasketError::MathOverflow)?;

        let config = &mut ctx.accounts.config;
        let mint_authority_bump = [config.mint_authority_bump];
        let signer_seeds: &[&[&[u8]]] = &[&[MINT_AUTHORITY_SEED, &mint_authority_bump]];
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.usdc_vault.to_account_info(),
                to: ctx.accounts.user_usdc.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(transfer_ctx, usdc_to_return)?;

        config.update_nav(total_assets_after, flex_supply_after)?;
        let nav_for_event = config.nav;

        emit!(RedeemEvent {
            redeemer: ctx.accounts.user.key(),
            flex_burned: params.amount,
            usdc_returned: usdc_to_return,
            nav: nav_for_event,
            timestamp: current_time,
        });

        Ok(())
    }

    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        let config = &mut ctx.accounts.config;

        if let Some(new_admin) = params.new_admin {
            config.admin = new_admin;
        }
        if let Some(new_guardian) = params.new_guardian {
            config.guardian = new_guardian;
        }
        if let Some(new_emergency_admin) = params.new_emergency_admin {
            config.emergency_admin = new_emergency_admin;
        }
        if let Some(paused) = params.paused {
            config.paused = paused;
        }
        if let Some(max_deposit_amount) = params.max_deposit_amount {
            require!(max_deposit_amount > 0, BasketError::InvalidLimits);
            config.max_deposit_amount = max_deposit_amount;
        }
        if let Some(max_daily_deposit) = params.max_daily_deposit {
            require!(max_daily_deposit > 0, BasketError::InvalidLimits);
            config.max_daily_deposit = max_daily_deposit;
        }

        emit!(ConfigUpdatedEvent {
            admin: config.admin,
            guardian: config.guardian,
            emergency_admin: config.emergency_admin,
            paused: config.paused,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeBasketParams {
    pub guardian: Pubkey,
    pub emergency_admin: Pubkey,
    pub max_deposit_amount: u64,
    pub max_daily_deposit: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UpdateConfigParams {
    pub new_admin: Option<Pubkey>,
    pub new_guardian: Option<Pubkey>,
    pub new_emergency_admin: Option<Pubkey>,
    pub paused: Option<bool>,
    pub max_deposit_amount: Option<u64>,
    pub max_daily_deposit: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositUsdcParams {
    pub amount: u64,
    pub min_flex_out: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RedeemFlexParams {
    pub amount: u64,
    pub min_usdc_out: u64,
}

#[account]
pub struct BasketConfig {
    pub bump: u8,
    pub mint_authority_bump: u8,
    pub usdc_vault_bump: u8,
    pub usdt_vault_bump: u8,
    pub admin: Pubkey,
    pub guardian: Pubkey,
    pub emergency_admin: Pubkey,
    pub flex_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdt_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub usdt_vault: Pubkey,
    pub nav: u64,
    pub flex_supply_snapshot: u64,
    pub last_total_assets: u64,
    pub paused: bool,
    pub max_deposit_amount: u64,
    pub max_daily_deposit: u64,
    pub daily_deposit_volume: u64,
    pub last_deposit_day: i64,
}

impl BasketConfig {
    pub const SPACE: usize = 8 + 4 + (8 * 32) + (7 * 8) + 1;

    fn update_nav(&mut self, total_assets: u128, flex_supply: u128) -> Result<()> {
        require!(total_assets <= u64::MAX as u128, BasketError::MathOverflow);
        require!(flex_supply <= u64::MAX as u128, BasketError::MathOverflow);

        self.last_total_assets = total_assets as u64;
        self.flex_supply_snapshot = flex_supply as u64;

        self.nav = if flex_supply == 0 {
            DECIMAL_FACTOR as u64
        } else {
            // SECURITY: Prevent overflow by checking bounds before multiplication
            const MAX_SAFE_TOTAL_ASSETS: u128 = u128::MAX / DECIMAL_FACTOR;
            require!(
                total_assets <= MAX_SAFE_TOTAL_ASSETS,
                BasketError::TotalAssetsTooLarge
            );

            let nav = total_assets
                .checked_mul(DECIMAL_FACTOR)
                .ok_or(BasketError::MathOverflow)?
                .checked_div(flex_supply)
                .ok_or(BasketError::ZeroNav)?;

            // Additional bounds checking
            require!(nav <= u128::from(u64::MAX), BasketError::MathOverflow);
            nav as u64
        };

        Ok(())
    }

    fn check_daily_limit(&mut self, amount: u64, current_time: i64) -> Result<()> {
        let current_day = current_time / 86400;

        // SECURITY: Atomic daily limit check to prevent race conditions
        // This ensures the check and update happen together without interruption

        if self.last_deposit_day != current_day {
            // New day - reset daily volume atomically
            self.daily_deposit_volume = amount; // Set directly to current amount
            self.last_deposit_day = current_day;
        } else {
            // Same day - check and update atomically
            let new_daily_volume = self
                .daily_deposit_volume
                .checked_add(amount)
                .ok_or(BasketError::MathOverflow)?;

            require!(
                new_daily_volume <= self.max_daily_deposit,
                BasketError::DailyDepositLimitExceeded
            );

            self.daily_deposit_volume = new_daily_volume;
        }

        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn can_deposit(&self, amount: u64) -> Result<()> {
        require!(!self.is_paused(), BasketError::ContractPaused);
        require!(
            amount <= self.max_deposit_amount,
            BasketError::MaxDepositAmountExceeded
        );
        Ok(())
    }
}

fn total_assets(usdc_vault: &SplTokenAccount, usdt_vault: &SplTokenAccount) -> Result<u128> {
    let total = (usdc_vault.amount as u128)
        .checked_add(usdt_vault.amount as u128)
        .ok_or(BasketError::MathOverflow)?;
    Ok(total)
}

#[derive(Accounts)]
pub struct InitializeBasket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = BasketConfig::SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, BasketConfig>,
    /// CHECK: PDA signer for mint and vault authorities
    #[account(seeds = [MINT_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub flex_mint: Account<'info, TokenMint>,
    pub usdc_mint: Account<'info, TokenMint>,
    pub usdt_mint: Account<'info, TokenMint>,
    #[account(
        init,
        payer = payer,
        seeds = [VAULT_SEED, usdc_mint.key().as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = mint_authority
    )]
    pub usdc_vault: Account<'info, SplTokenAccount>,
    #[account(
        init,
        payer = payer,
        seeds = [VAULT_SEED, usdt_mint.key().as_ref()],
        bump,
        token::mint = usdt_mint,
        token::authority = mint_authority
    )]
    pub usdt_vault: Account<'info, SplTokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositUsdc<'info> {
    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, BasketConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = user_usdc.owner == user.key(),
        constraint = user_usdc.mint == config.usdc_mint
    )]
    pub user_usdc: Account<'info, SplTokenAccount>,
    #[account(
        mut,
        constraint = user_flex.owner == user.key(),
        constraint = user_flex.mint == config.flex_mint
    )]
    pub user_flex: Account<'info, SplTokenAccount>,
    #[account(address = config.usdc_mint)]
    pub usdc_mint: Account<'info, TokenMint>,
    #[account(mut, address = config.usdc_vault)]
    pub usdc_vault: Account<'info, SplTokenAccount>,
    #[account(address = config.usdt_vault)]
    pub usdt_vault: Account<'info, SplTokenAccount>,
    #[account(mut, address = config.flex_mint)]
    pub flex_mint: Account<'info, TokenMint>,
    /// CHECK: PDA signer for mint and vault authorities
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RedeemFlex<'info> {
    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, BasketConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = user_flex.owner == user.key(),
        constraint = user_flex.mint == config.flex_mint
    )]
    pub user_flex: Account<'info, SplTokenAccount>,
    #[account(
        mut,
        constraint = user_usdc.owner == user.key(),
        constraint = user_usdc.mint == config.usdc_mint
    )]
    pub user_usdc: Account<'info, SplTokenAccount>,
    #[account(address = config.usdc_mint)]
    pub usdc_mint: Account<'info, TokenMint>,
    #[account(mut, address = config.usdc_vault)]
    pub usdc_vault: Account<'info, SplTokenAccount>,
    #[account(address = config.usdt_vault)]
    pub usdt_vault: Account<'info, SplTokenAccount>,
    #[account(mut, address = config.flex_mint)]
    pub flex_mint: Account<'info, TokenMint>,
    /// CHECK: PDA signer for mint and vault authorities
    #[account(seeds = [MINT_AUTHORITY_SEED], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, has_one = admin, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, BasketConfig>,
}

#[event]
pub struct DepositEvent {
    pub depositor: Pubkey,
    pub usdc_amount: u64,
    pub flex_minted: u64,
    pub nav: u64,
    pub timestamp: i64,
}

#[event]
pub struct RedeemEvent {
    pub redeemer: Pubkey,
    pub flex_burned: u64,
    pub usdc_returned: u64,
    pub nav: u64,
    pub timestamp: i64,
}

#[event]
pub struct ConfigUpdatedEvent {
    pub admin: Pubkey,
    pub guardian: Pubkey,
    pub emergency_admin: Pubkey,
    pub paused: bool,
    pub timestamp: i64,
}

#[event]
pub struct ConfigInitializedEvent {
    pub admin: Pubkey,
    pub guardian: Pubkey,
    pub emergency_admin: Pubkey,
    pub flex_mint: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum BasketError {
    #[msg("Mint decimals must equal configured basket decimals")]
    InvalidMintDecimals,
    #[msg("Mint authority does not match the expected admin")]
    InvalidMintAuthority,
    #[msg("Amount must be positive")]
    AmountMustBePositive,
    #[msg("Operation would produce zero share change at current NAV")]
    AmountTooSmallForShare,
    #[msg("Mathematical overflow")]
    MathOverflow,
    #[msg("Total assets too large for safe NAV calculation")]
    TotalAssetsTooLarge,
    #[msg("Total assets cannot be zero when supply exists")]
    ZeroNav,
    #[msg("FLEX supply is zero")]
    ZeroSupply,
    #[msg("User does not hold sufficient balance")]
    InsufficientUserFunds,
    #[msg("Vault does not hold enough liquidity")]
    InsufficientVaultLiquidity,
    #[msg("Total assets are zero")]
    ZeroTotalAssets,
    #[msg("Bump not found in context")]
    BumpNotFound,
    #[msg("Contract is paused")]
    ContractPaused,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Price impact exceeds maximum allowed threshold")]
    ExcessivePriceImpact,
    #[msg("Single deposit amount exceeds maximum limit")]
    SingleDepositTooLarge,
    #[msg("Daily deposit limit exceeded")]
    DailyDepositLimitExceeded,
    #[msg("Maximum deposit amount exceeded")]
    MaxDepositAmountExceeded,
    #[msg("Invalid limits provided")]
    InvalidLimits,
}
