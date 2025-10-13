use anchor_lang::prelude::*;

declare_id!("StraTegy1111111111111111111111111111111111");

#[program]
pub mod strategy {
    use super::*;

    pub fn set_targets(_ctx: Context<UpdateStrategy>, _targets: TargetWeights) -> Result<()> {
        Ok(())
    }

    pub fn set_thresholds(_ctx: Context<UpdateStrategy>, _thresholds: DriftThreshold) -> Result<()> {
        Ok(())
    }

    pub fn set_caps(_ctx: Context<UpdateStrategy>, _caps: WeightCaps) -> Result<()> {
        Ok(())
    }

    pub fn set_oracle_values(
        _ctx: Context<UpdateStrategy>,
        _oracle_values: OracleSignals,
    ) -> Result<()> {
        Ok(())
    }
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

#[derive(Accounts)]
pub struct UpdateStrategy<'info> {
    // TODO: enforce admin authority and load strategy config PDA
    pub admin: Signer<'info>,
}
