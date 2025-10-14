use anchor_lang::{prelude::*, AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::{
    instruction::{self as token_instruction},
    processor::Processor as TokenProcessor,
};
use std::mem;

use basket::{self, InitializeBasketParams};

const USDC_DEPOSIT: u64 = 1_000_000; // 1 USDC (6 decimals)
const FLEX_REDEEM: u64 = 500_000; // 0.5 FLEX

async fn initialize_deposit_redeem_flow() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new("basket", basket::ID, basket::entry);
    program_test.add_program("spl_token", spl_token::id(), TokenProcessor::process);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create test accounts
    let admin = Keypair::new();
    let user = Keypair::new();
    let config_pda = Pubkey::find_program_address(&[b"basket-config"], &basket::ID).0;
    let mint_authority_pda = Pubkey::find_program_address(&[b"mint-authority"], &basket::ID).0;
    let flex_mint = Keypair::new();
    let usdc_mint = Keypair::new();
    let usdt_mint = Keypair::new();
    let usdc_vault_pda =
        Pubkey::find_program_address(&[b"vault", usdc_mint.pubkey().as_ref()], &basket::ID).0;
    let usdt_vault_pda =
        Pubkey::find_program_address(&[b"vault", usdt_mint.pubkey().as_ref()], &basket::ID).0;
    let user_usdc = Keypair::new();
    let user_flex = Keypair::new();

    // Fund accounts
    fund_account(&mut banks_client, &payer, &admin.pubkey(), 10_000_000_000).await?;
    fund_account(&mut banks_client, &payer, &user.pubkey(), 10_000_000_000).await?;

    // Create mints
    create_mint(&mut banks_client, &payer, &flex_mint, &admin, 6).await?;
    create_mint(&mut banks_client, &payer, &usdc_mint, &admin, 6).await?;
    create_mint(&mut banks_client, &payer, &usdt_mint, &admin, 6).await?;

    // Create token accounts
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

    // Mint initial tokens to user
    mint_tokens(
        &mut banks_client,
        &payer,
        &usdc_mint.pubkey(),
        &user_usdc.pubkey(),
        &admin,
        5_000_000,
    )
    .await?;

    // Initialize basket
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
                max_deposit_amount: 10_000_000,
                max_daily_deposit: 50_000_000,
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[init_ix], &[&admin]).await?;

    // Deposit USDC
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
            params: basket::DepositUsdcParams {
                amount: USDC_DEPOSIT,
                min_flex_out: 900_000, // 0.9 FLEX minimum
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[deposit_ix], &[&user]).await?;

    // Redeem FLEX
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
            params: basket::RedeemFlexParams {
                amount: FLEX_REDEEM,
                min_usdc_out: 400_000, // 0.4 USDC minimum
            },
        }
        .data(),
    };

    send_transaction(&mut banks_client, &payer, &[redeem_ix], &[&user]).await?;

    Ok(())
}

async fn fund_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recipient: &Pubkey,
    lamports: u64,
) -> Result<(), TransportError> {
    let instruction = token_instruction::transfer(&payer.pubkey(), recipient, lamports);
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
    let mint_len = 82; // Mint state length
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            rent.minimum_balance(mint_len),
            mint_len as u64,
            &spl_token::id(),
        ),
        token_instruction::initialize_mint(
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
    let account_len = 165; // Token account state length
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(account_len),
            account_len as u64,
            &spl_token::id(),
        ),
        token_instruction::initialize_account(&spl_token::id(), &account.pubkey(), mint, owner)
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
    let instruction = token_instruction::mint_to(
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

#[tokio::test]
async fn test_basket_program() {
    assert!(initialize_deposit_redeem_flow().await.is_ok());
}
