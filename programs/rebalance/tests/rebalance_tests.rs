use anchor_lang::{prelude::*, InstructionData, ToAccountMetas};
use rebalance;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    transport::TransportError,
};

#[tokio::test]
async fn test_rebalance_once() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new("rebalance", rebalance::ID, rebalance::entry);
    let (mut banks_client, payer, _) = program_test.start().await;

    let authority = Keypair::new();
    fund_account(&mut banks_client, &payer, &authority.pubkey(), 5_000_000_000).await?;

    let rebalance_ix = Instruction {
        program_id: rebalance::ID,
        accounts: rebalance::accounts::ExecuteRebalance {
            authority: authority.pubkey(),
        }
        .to_account_metas(None),
        data: rebalance::instruction::RebalanceOnce {}.data(),
    };

    send_transaction(&mut banks_client, &payer, &[rebalance_ix], &[&authority]).await?;

    Ok(())
}

#[tokio::test]
async fn test_pause_rebalancing() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new("rebalance", rebalance::ID, rebalance::entry);
    let (mut banks_client, payer, _) = program_test.start().await;

    let guardian = Keypair::new();
    fund_account(&mut banks_client, &payer, &guardian.pubkey(), 5_000_000_000).await?;

    let pause_ix = Instruction {
        program_id: rebalance::ID,
        accounts: rebalance::accounts::ToggleRebalancing {
            guardian: guardian.pubkey(),
        }
        .to_account_metas(None),
        data: rebalance::instruction::PauseRebalancing {}.data(),
    };

    send_transaction(&mut banks_client, &payer, &[pause_ix], &[&guardian]).await?;

    Ok(())
}

#[tokio::test]
async fn test_unpause_rebalancing() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new("rebalance", rebalance::ID, rebalance::entry);
    let (mut banks_client, payer, _) = program_test.start().await;

    let guardian = Keypair::new();
    fund_account(&mut banks_client, &payer, &guardian.pubkey(), 5_000_000_000).await?;

    let unpause_ix = Instruction {
        program_id: rebalance::ID,
        accounts: rebalance::accounts::ToggleRebalancing {
            guardian: guardian.pubkey(),
        }
        .to_account_metas(None),
        data: rebalance::instruction::UnpauseRebalancing {}.data(),
    };

    send_transaction(&mut banks_client, &payer, &[unpause_ix], &[&guardian]).await?;

    Ok(())
}

#[tokio::test]
async fn test_pause_and_unpause_sequence() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new("rebalance", rebalance::ID, rebalance::entry);
    let (mut banks_client, payer, _) = program_test.start().await;

    let guardian = Keypair::new();
    fund_account(&mut banks_client, &payer, &guardian.pubkey(), 5_000_000_000).await?;

    // Pause
    let pause_ix = Instruction {
        program_id: rebalance::ID,
        accounts: rebalance::accounts::ToggleRebalancing {
            guardian: guardian.pubkey(),
        }
        .to_account_metas(None),
        data: rebalance::instruction::PauseRebalancing {}.data(),
    };
    send_transaction(&mut banks_client, &payer, &[pause_ix], &[&guardian]).await?;

    // Unpause
    let unpause_ix = Instruction {
        program_id: rebalance::ID,
        accounts: rebalance::accounts::ToggleRebalancing {
            guardian: guardian.pubkey(),
        }
        .to_account_metas(None),
        data: rebalance::instruction::UnpauseRebalancing {}.data(),
    };
    send_transaction(&mut banks_client, &payer, &[unpause_ix], &[&guardian]).await?;

    Ok(())
}

// Helper functions

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