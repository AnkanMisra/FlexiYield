use anchor_lang::prelude::*;

const STRATEGY_CONFIG_SEED: &[u8] = b"strategy-config";
const BPS_DENOMINATOR: u16 = 10_000;

declare_id!("StraTegy1111111111111111111111111111111111");

#[program]
pub mod strategy {
    use super::*;

    pub fn initialize_strategy(
        ctx: Context<InitializeStrategy>,
        params: InitializeStrategyParams,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        let InitializeStrategyBumps { config: config_bump } = ctx.bumps;
        config.bump = config_bump;
        config.admin = ctx.accounts.admin.key();
        config.guardian = if params.guardian == Pubkey::default() {
            ctx.accounts.admin.key()
        } else {
            params.guardian
        };

        // Initialize with default 50/50 target weights
        config.target_weights = TargetWeights {
            usdc_weight_bps: 5_000,
            usdt_weight_bps: 5_000,
        };
        config.drift_threshold = DriftThreshold { bps: 500 }; // 5%
        config.weight_caps = WeightCaps {
            usdc_cap_bps: 8_000, // 80% cap
            usdt_cap_bps: 8_000, // 80% cap
        };
        config.oracle_signals = OracleSignals {
            usdc_apy_bps: 0,
            usdt_apy_bps: 0,
            usdc_peg_stable: true,
            usdt_peg_stable: true,
        };
        config.last_updated = Clock::get()?.unix_timestamp;

        emit!(StrategyInitializedEvent {
            admin: config.admin,
            guardian: config.guardian,
        });

        Ok(())
    }

    pub fn set_targets(
        ctx: Context<UpdateStrategy>,
        targets: TargetWeights,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        // Validate target weights sum to 10,000 bps (100%)
        require!(
            targets.usdc_weight_bps.saturating_add(targets.usdt_weight_bps) == BPS_DENOMINATOR,
            StrategyError::InvalidTargetWeights
        );

        // Validate against caps
        require!(
            targets.usdc_weight_bps <= config.weight_caps.usdc_cap_bps,
            StrategyError::TargetExceedsCap
        );
        require!(
            targets.usdt_weight_bps <= config.weight_caps.usdt_cap_bps,
            StrategyError::TargetExceedsCap
        );

        config.target_weights = targets;
        config.last_updated = Clock::get()?.unix_timestamp;

        emit!(TargetsUpdatedEvent {
            usdc_weight_bps: targets.usdc_weight_bps,
            usdt_weight_bps: targets.usdt_weight_bps,
        });

        Ok(())
    }

    pub fn set_thresholds(
        ctx: Context<UpdateStrategy>,
        threshold: DriftThreshold,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        // Validate threshold is reasonable (0-1000 bps = 0-10%)
        require!(
            threshold.bps <= 1_000,
            StrategyError::InvalidThreshold
        );

        config.drift_threshold = threshold;
        config.last_updated = Clock::get()?.unix_timestamp;

        emit!(ThresholdUpdatedEvent { bps: threshold.bps });

        Ok(())
    }

    pub fn set_caps(ctx: Context<UpdateStrategy>, caps: WeightCaps) -> Result<()> {
        let config = &mut ctx.accounts.config;

        // Validate caps are reasonable (0-10,000 bps = 0-100%)
        require!(
            caps.usdc_cap_bps <= BPS_DENOMINATOR && caps.usdt_cap_bps <= BPS_DENOMINATOR,
            StrategyError::InvalidCaps
        );

        // Validate current targets don't exceed new caps
        require!(
            config.target_weights.usdc_weight_bps <= caps.usdc_cap_bps,
            StrategyError::TargetExceedsCap
        );
        require!(
            config.target_weights.usdt_weight_bps <= caps.usdt_cap_bps,
            StrategyError::TargetExceedsCap
        );

        config.weight_caps = caps;
        config.last_updated = Clock::get()?.unix_timestamp;

        emit!(CapsUpdatedEvent {
            usdc_cap_bps: caps.usdc_cap_bps,
            usdt_cap_bps: caps.usdt_cap_bps,
        });

        Ok(())
    }

    pub fn set_oracle_values(
        ctx: Context<UpdateStrategy>,
        oracle_values: OracleSignals,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        // Validate APY values are reasonable (-50,000 to 50,000 bps = -500% to 500%)
        require!(
            oracle_values.usdc_apy_bps.abs() <= 50_000,
            StrategyError::InvalidApyValue
        );
        require!(
            oracle_values.usdt_apy_bps.abs() <= 50_000,
            StrategyError::InvalidApyValue
        );

        config.oracle_signals = oracle_values;
        config.last_updated = Clock::get()?.unix_timestamp;

        emit!(OracleUpdatedEvent {
            usdc_apy_bps: oracle_values.usdc_apy_bps,
            usdt_apy_bps: oracle_values.usdt_apy_bps,
            usdc_peg_stable: oracle_values.usdc_peg_stable,
            usdt_peg_stable: oracle_values.usdt_peg_stable,
        });

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeStrategyParams {
    pub guardian: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TargetWeights {
    pub usdc_weight_bps: u16,
    pub usdt_weight_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DriftThreshold {
    pub bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct WeightCaps {
    pub usdc_cap_bps: u16,
    pub usdt_cap_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct OracleSignals {
    pub usdc_apy_bps: i32,
    pub usdt_apy_bps: i32,
    pub usdc_peg_stable: bool,
    pub usdt_peg_stable: bool,
}

#[account]
pub struct StrategyConfig {
    pub bump: u8,
    pub admin: Pubkey,
    pub guardian: Pubkey,
    pub target_weights: TargetWeights,
    pub drift_threshold: DriftThreshold,
    pub weight_caps: WeightCaps,
    pub oracle_signals: OracleSignals,
    pub last_updated: i64,
}

impl StrategyConfig {
    pub const SPACE: usize = 8 + 1 + (2 * 32) + 4 + 2 + 2 + 2 + 2 + 2 + 2 + 4 + 1 + 1 + 8;
}

#[derive(Accounts)]
pub struct InitializeStrategy<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = StrategyConfig::SPACE,
        seeds = [STRATEGY_CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, StrategyConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateStrategy<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin,
        seeds = [STRATEGY_CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, StrategyConfig>,
}

#[event]
pub struct StrategyInitializedEvent {
    pub admin: Pubkey,
    pub guardian: Pubkey,
}

#[event]
pub struct TargetsUpdatedEvent {
    pub usdc_weight_bps: u16,
    pub usdt_weight_bps: u16,
}

#[event]
pub struct ThresholdUpdatedEvent {
    pub bps: u16,
}

#[event]
pub struct CapsUpdatedEvent {
    pub usdc_cap_bps: u16,
    pub usdt_cap_bps: u16,
}

#[event]
pub struct OracleUpdatedEvent {
    pub usdc_apy_bps: i32,
    pub usdt_apy_bps: i32,
    pub usdc_peg_stable: bool,
    pub usdt_peg_stable: bool,
}

#[error_code]
pub enum StrategyError {
    #[msg("Target weights must sum to 10,000 bps (100%)")]
    InvalidTargetWeights,
    #[msg("Target weight exceeds configured cap")]
    TargetExceedsCap,
    #[msg("Drift threshold must be between 0 and 1,000 bps (0-10%)")]
    InvalidThreshold,
    #[msg("Weight caps must be between 0 and 10,000 bps (0-100%)")]
    InvalidCaps,
    #[msg("APY values must be between -50,000 and 50,000 bps (-500% to 500%)")]
    InvalidApyValue,
}
