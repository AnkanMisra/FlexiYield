use anchor_lang::{prelude::*, AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    transport::TransportError,
};
use strategy::{
    self, DriftThreshold, InitializeStrategyParams, OracleSignals, TargetWeights, WeightCaps,
};

const STRATEGY_CONFIG_SEED: &[u8] = b"strategy-config";

#[tokio::test]
async fn test_initialize_strategy() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new("strategy", strategy::ID, strategy::entry);
    let (mut banks_client, payer, _) = program_test.start().await;

    let admin = Keypair::new();
    fund_account(&mut banks_client, &payer, &admin.pubkey(), 5_000_000_000).await?;

    let (config_pda, _) = Pubkey::find_program_address(&[STRATEGY_CONFIG_SEED], &strategy::ID);

    let init_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::InitializeStrategy {
            payer: payer.pubkey(),
            admin: admin.pubkey(),
            config: config_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: strategy::instruction::InitializeStrategy {
            params: InitializeStrategyParams {
                guardian: admin.pubkey(),
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[init_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.admin, admin.pubkey());
    assert_eq!(config.guardian, admin.pubkey());
    assert_eq!(config.target_weights.usdc_weight_bps, 5_000);
    assert_eq!(config.target_weights.usdt_weight_bps, 5_000);
    assert_eq!(config.drift_threshold.bps, 500);
    assert_eq!(config.weight_caps.usdc_cap_bps, 8_000);
    assert_eq!(config.weight_caps.usdt_cap_bps, 8_000);
    assert_eq!(config.oracle_signals.usdc_apy_bps, 0);
    assert_eq!(config.oracle_signals.usdt_apy_bps, 0);
    assert_eq!(config.oracle_signals.usdc_peg_stable, true);
    assert_eq!(config.oracle_signals.usdt_peg_stable, true);

    Ok(())
}

#[tokio::test]
async fn test_set_valid_targets() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let new_targets = TargetWeights {
        usdc_weight_bps: 6_000,
        usdt_weight_bps: 4_000,
    };

    let set_targets_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetTargets {
            targets: new_targets.clone(),
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[set_targets_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.target_weights.usdc_weight_bps, 6_000);
    assert_eq!(config.target_weights.usdt_weight_bps, 4_000);

    Ok(())
}

#[tokio::test]
async fn test_set_targets_not_summing_to_100_percent_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let invalid_targets = TargetWeights {
        usdc_weight_bps: 6_000,
        usdt_weight_bps: 5_000, // Sum is 11,000 instead of 10,000
    };

    let set_targets_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetTargets {
            targets: invalid_targets,
        }
        .data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_targets_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail when targets don't sum to 100%");

    Ok(())
}

#[tokio::test]
async fn test_set_targets_exceeding_caps_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    // Default caps are 8000 bps (80%)
    let invalid_targets = TargetWeights {
        usdc_weight_bps: 8_500, // Exceeds cap
        usdt_weight_bps: 1_500,
    };

    let set_targets_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetTargets {
            targets: invalid_targets,
        }
        .data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_targets_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail when targets exceed caps");

    Ok(())
}

#[tokio::test]
async fn test_set_valid_thresholds() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let new_threshold = DriftThreshold { bps: 750 }; // 7.5%

    let set_threshold_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetThresholds {
            threshold: new_threshold.clone(),
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[set_threshold_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.drift_threshold.bps, 750);

    Ok(())
}

#[tokio::test]
async fn test_set_threshold_exceeding_max_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let invalid_threshold = DriftThreshold { bps: 1_001 }; // Over 10%

    let set_threshold_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetThresholds {
            threshold: invalid_threshold,
        }
        .data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_threshold_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail when threshold exceeds 10%");

    Ok(())
}

#[tokio::test]
async fn test_set_valid_caps() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let new_caps = WeightCaps {
        usdc_cap_bps: 9_000,
        usdt_cap_bps: 9_000,
    };

    let set_caps_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetCaps { caps: new_caps.clone() }.data(),
    };

    send_transaction(&mut banks_client, &payer, &[set_caps_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.weight_caps.usdc_cap_bps, 9_000);
    assert_eq!(config.weight_caps.usdt_cap_bps, 9_000);

    Ok(())
}

#[tokio::test]
async fn test_set_caps_below_current_targets_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    // First set high targets
    let high_targets = TargetWeights {
        usdc_weight_bps: 7_000,
        usdt_weight_bps: 3_000,
    };

    let set_targets_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetTargets {
            targets: high_targets,
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[set_targets_ix], &[&admin]).await?;

    // Now try to set caps below current targets
    let low_caps = WeightCaps {
        usdc_cap_bps: 6_000, // Below current 7000 target
        usdt_cap_bps: 9_000,
    };

    let set_caps_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetCaps { caps: low_caps }.data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_caps_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail when caps are below current targets");

    Ok(())
}

#[tokio::test]
async fn test_set_caps_exceeding_100_percent_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let invalid_caps = WeightCaps {
        usdc_cap_bps: 10_001, // Over 100%
        usdt_cap_bps: 9_000,
    };

    let set_caps_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetCaps { caps: invalid_caps }.data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_caps_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail when caps exceed 100%");

    Ok(())
}

#[tokio::test]
async fn test_set_valid_oracle_values() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let oracle_values = OracleSignals {
        usdc_apy_bps: 500,  // 5% APY
        usdt_apy_bps: -200, // -2% APY
        usdc_peg_stable: true,
        usdt_peg_stable: false,
    };

    let set_oracle_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetOracleValues {
            oracle_values: oracle_values.clone(),
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[set_oracle_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.oracle_signals.usdc_apy_bps, 500);
    assert_eq!(config.oracle_signals.usdt_apy_bps, -200);
    assert_eq!(config.oracle_signals.usdc_peg_stable, true);
    assert_eq!(config.oracle_signals.usdt_peg_stable, false);

    Ok(())
}

#[tokio::test]
async fn test_set_oracle_values_with_extreme_apy_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let invalid_oracle = OracleSignals {
        usdc_apy_bps: 50_001, // Over 500% APY
        usdt_apy_bps: 0,
        usdc_peg_stable: true,
        usdt_peg_stable: true,
    };

    let set_oracle_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetOracleValues {
            oracle_values: invalid_oracle,
        }
        .data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_oracle_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail with extreme APY values");

    Ok(())
}

#[tokio::test]
async fn test_set_oracle_values_with_negative_extreme_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    let invalid_oracle = OracleSignals {
        usdc_apy_bps: 0,
        usdt_apy_bps: -50_001, // Under -500% APY
        usdc_peg_stable: true,
        usdt_peg_stable: true,
    };

    let set_oracle_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetOracleValues {
            oracle_values: invalid_oracle,
        }
        .data(),
    };

    let result = send_transaction(&mut banks_client, &payer, &[set_oracle_ix], &[&admin]).await;
    assert!(result.is_err(), "Should fail with extreme negative APY values");

    Ok(())
}

#[tokio::test]
async fn test_boundary_values() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda) = setup_initialized_strategy().await?;

    // Test max valid threshold (1000 bps = 10%)
    let max_threshold = DriftThreshold { bps: 1_000 };
    let set_threshold_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetThresholds {
            threshold: max_threshold,
        }
        .data(),
    };
    send_transaction(&mut banks_client, &payer, &[set_threshold_ix], &[&admin]).await?;

    // Test min valid threshold (0 bps)
    let min_threshold = DriftThreshold { bps: 0 };
    let set_threshold_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetThresholds {
            threshold: min_threshold,
        }
        .data(),
    };
    send_transaction(&mut banks_client, &payer, &[set_threshold_ix], &[&admin]).await?;

    // Test max valid caps (10000 bps = 100%)
    let max_caps = WeightCaps {
        usdc_cap_bps: 10_000,
        usdt_cap_bps: 10_000,
    };
    let set_caps_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetCaps { caps: max_caps }.data(),
    };
    send_transaction(&mut banks_client, &payer, &[set_caps_ix], &[&admin]).await?;

    // Test extreme valid oracle values (±50000 bps = ±500%)
    let extreme_oracle_positive = OracleSignals {
        usdc_apy_bps: 50_000,
        usdt_apy_bps: 0,
        usdc_peg_stable: true,
        usdt_peg_stable: true,
    };
    let set_oracle_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetOracleValues {
            oracle_values: extreme_oracle_positive,
        }
        .data(),
    };
    send_transaction(&mut banks_client, &payer, &[set_oracle_ix], &[&admin]).await?;

    let extreme_oracle_negative = OracleSignals {
        usdc_apy_bps: 0,
        usdt_apy_bps: -50_000,
        usdc_peg_stable: true,
        usdt_peg_stable: true,
    };
    let set_oracle_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::UpdateStrategy {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: strategy::instruction::SetOracleValues {
            oracle_values: extreme_oracle_negative,
        }
        .data(),
    };
    send_transaction(&mut banks_client, &payer, &[set_oracle_ix], &[&admin]).await?;

    Ok(())
}

// Helper functions

async fn setup_initialized_strategy() -> Result<(BanksClient, Keypair, Keypair, Pubkey), TransportError> {
    let mut program_test = ProgramTest::new("strategy", strategy::ID, strategy::entry);
    let (mut banks_client, payer, _) = program_test.start().await;

    let admin = Keypair::new();
    fund_account(&mut banks_client, &payer, &admin.pubkey(), 5_000_000_000).await?;

    let (config_pda, _) = Pubkey::find_program_address(&[STRATEGY_CONFIG_SEED], &strategy::ID);

    let init_ix = Instruction {
        program_id: strategy::ID,
        accounts: strategy::accounts::InitializeStrategy {
            payer: payer.pubkey(),
            admin: admin.pubkey(),
            config: config_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: strategy::instruction::InitializeStrategy {
            params: InitializeStrategyParams {
                guardian: admin.pubkey(),
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[init_ix], &[&admin]).await?;

    Ok((banks_client, payer, admin, config_pda))
}

async fn fund_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recipient: &Pubkey,
    lamports: u64,
) -> Result<(), TransportError> {
    let instruction = system_instruction::transfer(&payer.pubkey(), recipient, lamports);
    send_transaction(banks_client, payer, &[instruction], &[]).await
}

async fn send_transaction(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    instructions: &[Instruction],
    extra_signers: &[&Keypair],
) -> Result<(), TransportError> {
    let blockhash = banks_client.get_latest_blockhash().await?;
    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(payer);
    for signer in extra_signers {
        if signer.pubkey() != payer.pubkey() {
            signers.push(signer);
        }
    }
    let transaction = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &signers,
        blockhash,
    );
    banks_client.process_transaction(transaction).await
}

async fn fetch_config(banks_client: &mut BanksClient, config: &Pubkey) -> strategy::StrategyConfig {
    let account = banks_client
        .get_account(*config)
        .await
        .expect("fetch config")
        .expect("config account");
    let mut data_slice: &[u8] = &account.data;
    strategy::StrategyConfig::try_deserialize(&mut data_slice).expect("deserialize config")
}