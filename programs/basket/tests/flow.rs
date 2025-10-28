use anchor_lang::{prelude::*, AccountDeserialize, InstructionData, ToAccountMetas};
use basket::{self, DepositUsdcParams, InitializeBasketParams, RedeemFlexParams, UpdateConfigParams};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction, sysvar,
    transaction::Transaction,
    transport::TransportError,
};
use solana_sdk::program_pack::Pack;
use spl_token::state::{Account as TokenAccountState, Mint as MintState};

const CONFIG_SEED: &[u8] = b"basket-config";
const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority";
const VAULT_SEED: &[u8] = b"vault";
const DECIMALS: u8 = basket::BASKET_DECIMALS;
const USDC_DEPOSIT: u64 = 10_000_000; // 10 USDC with 6 decimals
const FLEX_REDEEM: u64 = 4_000_000; // redeem 4 FLEX
const NAV_MICROS: u64 = 1_000_000; // NAV precision matches basket program

#[tokio::test]
async fn initialize_deposit_redeem_flow() -> Result<(), TransportError> {
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
        20_000_000,
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
                max_deposit_amount: 50_000_000, // 50 USDC per tx
                max_daily_deposit: 500_000_000, // 500 USDC per day
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[init_ix], &[&admin]).await?;

    // Verify initialization
    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.admin, admin.pubkey());
    assert_eq!(config.guardian, admin.pubkey());
    assert_eq!(config.emergency_admin, admin.pubkey());
    assert_eq!(config.max_deposit_amount, 50_000_000);
    assert_eq!(config.max_daily_deposit, 500_000_000);
    assert_eq!(config.paused, false);
    assert_eq!(config.nav, NAV_MICROS);

    let deposit_ix = Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::DepositUsdc {
            config: config_pda,
            user: user.pubkey(),
            user_usdc: user_usdc.pubkey(),
            user_flex: user_flex.pubkey(),
            usdc_mint: usdc_mint.pubkey(),
            usdc_vault: usdc_vault_pda,
            usdt_vault: usdt_vault_pda,
            flex_mint: flex_mint.pubkey(),
            mint_authority: mint_authority_pda,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: basket::instruction::DepositUsdc {
            params: DepositUsdcParams {
                amount: USDC_DEPOSIT,
                min_flex_out: 9_000_000, // 9 FLEX minimum (90% of expected)
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await?;

    let config_after_deposit = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config_after_deposit.nav, NAV_MICROS);
    assert_eq!(config_after_deposit.flex_supply_snapshot, USDC_DEPOSIT);
    assert_eq!(config_after_deposit.last_total_assets, USDC_DEPOSIT);
    assert_eq!(config_after_deposit.daily_deposit_volume, USDC_DEPOSIT);

    let user_flex_after = fetch_token_account(&mut banks_client, &user_flex.pubkey()).await;
    assert_eq!(user_flex_after.amount, USDC_DEPOSIT);

    let redeem_ix = Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::RedeemFlex {
            config: config_pda,
            user: user.pubkey(),
            user_flex: user_flex.pubkey(),
            user_usdc: user_usdc.pubkey(),
            usdc_mint: usdc_mint.pubkey(),
            usdc_vault: usdc_vault_pda,
            usdt_vault: usdt_vault_pda,
            flex_mint: flex_mint.pubkey(),
            mint_authority: mint_authority_pda,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
        data: basket::instruction::RedeemFlex {
            params: RedeemFlexParams {
                amount: FLEX_REDEEM,
                min_usdc_out: 3_600_000, // 3.6 USDC minimum (90% of expected)
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[redeem_ix], &[&user]).await?;

    let config_after_redeem = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(
        config_after_redeem.flex_supply_snapshot,
        USDC_DEPOSIT - FLEX_REDEEM
    );
    assert_eq!(
        config_after_redeem.last_total_assets,
        USDC_DEPOSIT - FLEX_REDEEM
    );
    assert_eq!(config_after_redeem.nav, NAV_MICROS);

    let user_flex_final = fetch_token_account(&mut banks_client, &user_flex.pubkey()).await;
    assert_eq!(user_flex_final.amount, USDC_DEPOSIT - FLEX_REDEEM);

    let user_usdc_final = fetch_token_account(&mut banks_client, &user_usdc.pubkey()).await;
    assert_eq!(
        user_usdc_final.amount,
        20_000_000 - USDC_DEPOSIT + FLEX_REDEEM
    );

    let vault_final = fetch_token_account(&mut banks_client, &usdc_vault_pda).await;
    assert_eq!(vault_final.amount, USDC_DEPOSIT - FLEX_REDEEM);

    Ok(())
}

#[tokio::test]
async fn test_pause_and_unpause() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda, _, _, _, _) =
        setup_initialized_basket().await?;

    // Pause the contract
    let pause_ix = Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::UpdateConfig {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: basket::instruction::UpdateConfig {
            params: UpdateConfigParams {
                paused: Some(true),
                ..Default::default()
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[pause_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.paused, true);

    // Unpause the contract
    let unpause_ix = Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::UpdateConfig {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: basket::instruction::UpdateConfig {
            params: UpdateConfigParams {
                paused: Some(false),
                ..Default::default()
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[unpause_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.paused, false);

    Ok(())
}

#[tokio::test]
async fn test_update_config() -> Result<(), TransportError> {
    let (mut banks_client, payer, admin, config_pda, _, _, _, _) =
        setup_initialized_basket().await?;

    let new_guardian = Keypair::new();
    let new_emergency_admin = Keypair::new();

    let update_ix = Instruction {
        program_id: basket::ID,
        accounts: basket::accounts::UpdateConfig {
            admin: admin.pubkey(),
            config: config_pda,
        }
        .to_account_metas(None),
        data: basket::instruction::UpdateConfig {
            params: UpdateConfigParams {
                new_guardian: Some(new_guardian.pubkey()),
                new_emergency_admin: Some(new_emergency_admin.pubkey()),
                max_deposit_amount: Some(100_000_000),
                max_daily_deposit: Some(1_000_000_000),
                ..Default::default()
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[update_ix], &[&admin]).await?;

    let config = fetch_config(&mut banks_client, &config_pda).await;
    assert_eq!(config.guardian, new_guardian.pubkey());
    assert_eq!(config.emergency_admin, new_emergency_admin.pubkey());
    assert_eq!(config.max_deposit_amount, 100_000_000);
    assert_eq!(config.max_daily_deposit, 1_000_000_000);

    Ok(())
}

async fn setup_initialized_basket() -> Result<
    (
        BanksClient,
        Keypair,
        Keypair,
        Pubkey,
        Keypair,
        Keypair,
        Keypair,
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
    fund_account(&mut banks_client, &payer, &admin.pubkey(), 5_000_000_000).await?;

    let usdc_mint = Keypair::new();
    let usdt_mint = Keypair::new();
    let flex_mint = Keypair::new();

    create_mint(&mut banks_client, &payer, &usdc_mint, &admin, DECIMALS).await?;
    create_mint(&mut banks_client, &payer, &usdt_mint, &admin, DECIMALS).await?;
    create_mint(&mut banks_client, &payer, &flex_mint, &admin, DECIMALS).await?;

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
        config_pda,
        usdc_mint,
        usdt_mint,
        flex_mint,
        mint_authority_pda,
    ))
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

async fn fetch_token_account(
    banks_client: &mut BanksClient,
    address: &Pubkey,
) -> TokenAccountState {
    let account = banks_client
        .get_account(*address)
        .await
        .expect("fetch token")
        .expect("token account");
    TokenAccountState::unpack(&account.data).expect("deserialize token")
}