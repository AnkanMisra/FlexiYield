import { Connection, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { wallet } from './wallet';

const DEVNET_RPC = process.env.DEVNET_RPC || 'https://api.devnet.solana.com';

/**
 * Simple test script to validate wallet and connection functionality
 */
async function testConnection(): Promise<void> {
  console.log('🧪 Testing FlexiYield Scripts Setup');
  console.log('===================================');

  const connection = new Connection(DEVNET_RPC, 'confirmed');

  try {
    console.log(`📋 Wallet: ${wallet.publicKey.toBase58()}`);

    // Test connection
    console.log('🔗 Testing connection to devnet...');
    const slot = await connection.getSlot();
    console.log(`✅ Connected to devnet at slot: ${slot}`);

    // Check balance
    console.log('💰 Checking wallet balance...');
    const balance = await connection.getBalance(wallet.publicKey);
    console.log(`💳 SOL Balance: ${balance / LAMPORTS_PER_SOL} SOL`);

    // Test version
    console.log('🔍 Checking devnet version...');
    const version = await connection.getVersion();
    console.log(`✅ Solana version: ${version['solana-core']}`);

    console.log('\n🎉 All tests passed! Scripts are ready to use.');
    console.log('🚀 Run "pnpm scripts:demo:setup" for complete MVP setup');

  } catch (error) {
    console.error('❌ Test failed:', error);
    process.exit(1);
  }
}

// Execute if run directly
if (require.main === module) {
  testConnection()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error('❌ Test failed:', error);
      process.exit(1);
    });
}

export { testConnection };
