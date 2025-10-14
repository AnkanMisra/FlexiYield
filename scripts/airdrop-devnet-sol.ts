import { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { wallet } from './wallet';

const DEVNET_RPC = process.env.DEVNET_RPC || 'https://api.devnet.solana.com';
const AIRDROP_AMOUNT_SOL = 2;

/**
 * Airdrops SOL to the configured wallet on devnet
 */
async function airdropDevnetSol(): Promise<void> {
  console.log('🚀 Requesting SOL airdrop on devnet...');

  const connection = new Connection(DEVNET_RPC, 'confirmed');

  try {
    const balance = await connection.getBalance(wallet.publicKey);
    console.log(`Current balance: ${balance / LAMPORTS_PER_SOL} SOL`);

    if (balance >= LAMPORTS_PER_SOL) {
      console.log('✅ Sufficient SOL already available');
      return;
    }

    console.log(`Requesting ${AIRDROP_AMOUNT_SOL} SOL airdrop...`);
    const airdropSignature = await connection.requestAirdrop(
      wallet.publicKey,
      AIRDROP_AMOUNT_SOL * LAMPORTS_PER_SOL
    );

    console.log(`Airdrop signature: ${airdropSignature}`);
    console.log('⏳ Waiting for confirmation...');

    await connection.confirmTransaction(airdropSignature, 'confirmed');

    const newBalance = await connection.getBalance(wallet.publicKey);
    console.log(`✅ New balance: ${newBalance / LAMPORTS_PER_SOL} SOL`);

  } catch (error) {
    console.error('❌ Airdrop failed:', error);
    process.exit(1);
  }
}

// Execute if run directly
if (require.main === module) {
  airdropDevnetSol()
    .then(() => {
      console.log('✅ Airdrop completed successfully');
      process.exit(0);
    })
    .catch((error) => {
      console.error('❌ Airdrop failed:', error);
      process.exit(1);
    });
}

export { airdropDevnetSol };
