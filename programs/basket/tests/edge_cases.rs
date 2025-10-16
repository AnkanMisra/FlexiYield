use anchor_lang::{prelude::*, AccountDeserialize, InstructionData, ToAccountMetas};
use basket::{self, DepositUsdcParams, InitializeBasketParams, RedeemFlexParams};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction, sysvar,
    transaction::Transaction,
    transport::TransportError,
    program_pack::Pack,
};
use spl_token::state::{Account as TokenAccountState, Mint as MintState};

const CONFIG_SEED: &[u8] = b"basket-config";
const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority";
const VAULT_SEED: &[u8] = b"vault";
const DECIMALS: u8 = basket::BASKET_DECIMALS;

#[tokio::test]
async fn test_deposit_when_paused_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // Pause the contract
    let pause_ix = create_update_config_ix(&admin, &config_pda, true);
    send_transaction(&mut banks_client, &payer, &[pause_ix], &[&admin]).await?;

    // Try to deposit - should fail
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        1_000_000,
        900_000,
    );

    let result = send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await;
    assert!(result.is_err(), "Deposit should fail when paused");

    Ok(())
}

#[tokio::test]
async fn test_redeem_when_paused_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // First make a deposit
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        5_000_000,
        4_500_000,
    );
    send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await?;

    // Pause the contract
    let pause_ix = create_update_config_ix(&admin, &config_pda, true);
    send_transaction(&mut banks_client, &payer, &[pause_ix], &[&admin]).await?;

    // Try to redeem - should fail
    let redeem_ix = create_redeem_ix(
        &user,
        &config_pda,
        &user_flex,
        &user_usdc,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        1_000_000,
        900_000,
    );

    let result = send_transaction(&mut banks_client, &payer, &[redeem_ix], &[&user]).await;
    assert!(result.is_err(), "Redeem should fail when paused");

    Ok(())
}

#[tokio::test]
async fn test_deposit_exceeding_max_amount_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // Try to deposit more than max_deposit_amount (50 USDC)
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        51_000_000, // 51 USDC
        45_000_000,
    );

    let result = send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await;
    assert!(result.is_err(), "Deposit should fail when exceeding max amount");

    Ok(())
}

#[tokio::test]
async fn test_daily_deposit_limit() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // Make multiple deposits that stay under individual limit but exceed daily limit
    // max_daily_deposit is 500 USDC
    for _ in 0..10 {
        let deposit_ix = create_deposit_ix(
            &user,
            &config_pda,
            &user_usdc,
            &user_flex,
            &usdc_mint,
            &usdc_vault_pda,
            &usdt_vault_pda,
            &flex_mint,
            &mint_authority_pda,
            50_000_000, // 50 USDC each
            45_000_000,
        );
        send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await?;
    }

    // This deposit should fail as it exceeds daily limit
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        1_000_000,
        900_000,
    );

    let result = send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await;
    assert!(result.is_err(), "Deposit should fail when exceeding daily limit");

    Ok(())
}

#[tokio::test]
async fn test_slippage_protection_on_deposit() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // Try to deposit with unreasonably high min_flex_out
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        10_000_000, // 10 USDC
        20_000_000, // Expect 20 FLEX (impossible 1:2 ratio)
    );

    let result = send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await;
    assert!(result.is_err(), "Deposit should fail with excessive min_flex_out");

    Ok(())
}

#[tokio::test]
async fn test_slippage_protection_on_redeem() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // First make a deposit
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        10_000_000,
        9_000_000,
    );
    send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await?;

    // Try to redeem with unreasonably high min_usdc_out
    let redeem_ix = create_redeem_ix(
        &user,
        &config_pda,
        &user_flex,
        &user_usdc,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        5_000_000, // 5 FLEX
        10_000_000, // Expect 10 USDC (impossible 1:2 ratio)
    );

    let result = send_transaction(&mut banks_client, &payer, &[redeem_ix], &[&user]).await;
    assert!(result.is_err(), "Redeem should fail with excessive min_usdc_out");

    Ok(())
}

#[tokio::test]
async fn test_zero_amount_deposit_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        0, // Zero amount
        0,
    );

    let result = send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await;
    assert!(result.is_err(), "Zero amount deposit should fail");

    Ok(())
}

#[tokio::test]
async fn test_zero_amount_redeem_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // First make a deposit
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        10_000_000,
        9_000_000,
    );
    send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await?;

    let redeem_ix = create_redeem_ix(
        &user,
        &config_pda,
        &user_flex,
        &user_usdc,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        0, // Zero amount
        0,
    );

    let result = send_transaction(&mut banks_client, &payer, &[redeem_ix], &[&user]).await;
    assert!(result.is_err(), "Zero amount redeem should fail");

    Ok(())
}

#[tokio::test]
async fn test_insufficient_user_funds_fails() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // Try to deposit more than user has (user only has 100 USDC)
    let deposit_ix = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        150_000_000, // 150 USDC (more than user has)
        135_000_000,
    );

    let result = send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await;
    assert!(result.is_err(), "Deposit should fail with insufficient funds");

    Ok(())
}

#[tokio::test]
async fn test_nav_calculation_with_multiple_operations() -> Result<(), TransportError> {
    let (mut banks_client, payer, _, user, config_pda, user_usdc, user_flex, usdc_vault_pda, usdt_vault_pda, usdc_mint, flex_mint, mint_authority_pda) =
        setup_test_environment().await?;

    // First deposit
    let deposit1 = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        10_000_000,
        9_000_000,
    );
    send_transaction(&mut banks_client, &payer, &[deposit1], &[&user]).await?;

    let config_after_first = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config_after_first.nav, 1_000_000);

    // Second deposit
    let deposit2 = create_deposit_ix(
        &user,
        &config_pda,
        &user_usdc,
        &user_flex,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        20_000_000,
        18_000_000,
    );
    send_transaction(&mut banks_client, &payer, &[deposit2], &[&user]).await?;

    let config_after_second = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config_after_second.nav, 1_000_000);
    assert_eq!(config_after_second.last_total_assets, 30_000_000);

    // Redeem half
    let redeem = create_redeem_ix(
        &user,
        &config_pda,
        &user_flex,
        &user_usdc,
        &usdc_mint,
        &usdc_vault_pda,
        &usdt_vault_pda,
        &flex_mint,
        &mint_authority_pda,
        15_000_000,
        13_500_000,
    );
    send_transaction(&mut banks_client, &payer, &[redeem], &[&user]).await?;

    let config_final = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config_final.nav, 1_000_000);
    assert_eq!(config_final.last_total_assets, 15_000_000);
    assert_eq!(config_final.flex_supply_snapshot, 15_000_000);

    Ok(())
}

// Helper functions

async fn setup_test_environment() -> Result<
    (
        BanksClient,
        Keypair,
        Keypair,
        Keypair,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ),
    TransportError,
> {
    let mut program_test = ProgramTest::new("basket", basket::ID, basket::entry);
    program_test.add_program(
        "spl_token",
        spl_token::id(),
        processor!(spl_token::processor::Processor::process),
    );

    let (mut banks_client, payer, _) = program_test.start().await;

    let admin = Keypair::new();
    let user = Keypair::new();

    fund_account(&mut banks_client, &payer, &admin.pubkey(), 5_000_000_000).await?;
    fund_account(&mut banks_client, &payer, &user.pubkey(), 5_000_000_000).await?;

    let usdc_mint = Keypair::new();
    let usdt_mint = Keypair::new();
    let flex_mint = Keypair::new();

    create_mint(&mut banks_client, &payer, &usdc_mint, &admin, DECIMALS).await?;
    create_mint(&mut banks_client, &payer, &usdt_mint, &admin, DECIMALS).await?;
    create_mint(&mut banks_client, &payer, &flex_mint, &admin, DECIMALS).await?;

    let user_usdc = Keypair::new();
    let user_flex = Keypair::new();

    create_token_account(
        &mut banks_client,
        &payer,
        &user_usdc,
        &usdc_mint.pubkey(),
        &user.pubkey(),
    )
    .await?;
    create_token_account(
        &mut banks_client,
        &payer,
        &user_flex,
        &flex_mint.pubkey(),
        &user.pubkey(),
    )
    .await?;

    mint_tokens(
        &mut banks_client,
        &payer,
        &usdc_mint.pubkey(),
        &user_usdc.pubkey(),
        &admin,
        100_000_000, // 100 USDC
    )
    .await?;

    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &basket::ID);
    let (mint_authority_pda, _) = Pubkey::find_program_address(&[MINT_AUTHORITY_SEED], &basket::ID);
    let (usdc_vault_pda, _) =
        Pubkey::find_program_address(&[VAULT_SEED, usdc_mint.pubkey().as_ref()], &basket::ID);
    let (usdt_vault_pda, _) =
        Pubkey::find_program_address(&[VAULT_SEED, usdt_mint.pubkey().as_ref()], &basket::ID);

    let init_ix = Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::InitializeBasket {
            payer: payer.pubkey(),
            admin: admin.pubkey(),
            config: config_pda,
            mint_authority: mint_authority_pda,
            flex_mint: flex_mint.pubkey(),
            usdc_mint: usdc_mint.pubkey(),
            usdt_mint: usdt_mint.pubkey(),
            usdc_vault: usdc_vault_pda,
            usdt_vault: usdt_vault_pda,
            token_program: spl_token::id(),
            system_program: system_program::ID,
            rent: sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: basket::instruction::InitializeBasket {
            params: InitializeBasketParams {
                guardian: admin.pubkey(),
                emergency_admin: admin.pubkey(),
                max_deposit_amount: 50_000_000,
                max_daily_deposit: 500_000_000,
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[init_ix], &[&admin]).await?;

    Ok((
        banks_client,
        payer,
        admin,
        user,
        config_pda,
        user_usdc.pubkey(),
        user_flex.pubkey(),
        usdc_vault_pda,
        usdt_vault_pda,
        usdc_mint.pubkey(),
        flex_mint.pubkey(),
        mint_authority_pda,
    ))
}

fn create_deposit_ix(
    user: &Keypair,
    config: &Pubkey,
    user_usdc: &Pubkey,
    user_flex: &Pubkey,
    usdc_mint: &Pubkey,
    usdc_vault: &Pubkey,
    usdt_vault: &Pubkey,
    flex_mint: &Pubkey,
    mint_authority: &Pubkey,
    amount: u64,
    min_flex_out: u64,
) -> Instruction {
    Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::DepositUsdc {
            config: *config,
            user: user.pubkey(),
            user_usdc: *user_usdc,
            user_flex: *user_flex,
            usdc_mint: *usdc_mint,
            usdc_vault: *usdc_vault,
            usdt_vault: *usdt_vault,
            flex_mint: *flex_mint,
            mint_authority: *mint_authority,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: basket::instruction::DepositUsdc {
            params: DepositUsdcParams { amount, min_flex_out },
        }
        .data(),
    }
}

fn create_redeem_ix(
    user: &Keypair,
    config: &Pubkey,
    user_flex: &Pubkey,
    user_usdc: &Pubkey,
    usdc_mint: &Pubkey,
    usdc_vault: &Pubkey,
    usdt_vault: &Pubkey,
    flex_mint: &Pubkey,
    mint_authority: &Pubkey,
    amount: u64,
    min_usdc_out: u64,
) -> Instruction {
    Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::RedeemFlex {
            config: *config,
            user: user.pubkey(),
            user_flex: *user_flex,
            user_usdc: *user_usdc,
            usdc_mint: *usdc_mint,
            usdc_vault: *usdc_vault,
            usdt_vault: *usdt_vault,
            flex_mint: *flex_mint,
            mint_authority: *mint_authority,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: basket::instruction::RedeemFlex {
            params: RedeemFlexParams { amount, min_usdc_out },
        }
        .data(),
    }
}

fn create_update_config_ix(admin: &Keypair, config: &Pubkey, paused: bool) -> Instruction {
    Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::UpdateConfig {
            admin: admin.pubkey(),
            config: *config,
        }
        .to_account_metas(None),
        data: basket::instruction::UpdateConfig {
            params: basket::UpdateConfigParams {
                paused: Some(paused),
                ..Default::default()
            },
        }
        .data(),
    }
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

async fn create_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    mint: &Keypair,
    authority: &Keypair,
    decimals: u8,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await?;
    let mint_len = MintState::LEN;
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rent.minimum_balance(mint_len),
            mint_len as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            &authority.pubkey(),
            Some(&authority.pubkey()),
            decimals,
        )
        .expect("initialize mint"),
    ];
    send_transaction(banks_client, payer, &instructions, &[mint, authority]).await
}

async fn create_token_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    account: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await?;
    let account_len = TokenAccountState::LEN;
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_len),
            account_len as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_account(
            &spl_token::id(),
            &account.pubkey(),
            mint,
            owner,
        )
        .expect("initialize account"),
    ];
    send_transaction(banks_client, payer, &instructions, &[account]).await
}

async fn mint_tokens(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let instruction = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        destination,
        &authority.pubkey(),
        &[],
        amount,
    )
    .expect("mint to");
    send_transaction(banks_client, payer, &[instruction], &[authority]).await
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

async fn fetch_config(banks_client: &mut BanksClient, config: &Pubkey) -> basket::BasketConfig {
    let account = banks_client
        .get_account(*config)
        .await
        .expect("fetch config")
        .expect("config account");
    let mut data_slice: &[u8] = &account.data;
    basket::BasketConfig::try_deserialize(&mut data_slice).expect("deserialize config")
}