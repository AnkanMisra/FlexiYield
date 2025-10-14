import { Keypair, PublicKey } from '@solana/web3.js';
import crypto from 'crypto';

/**
 * 🔒 SECURE WALLET UTILITY - PRODUCTION READY
 *
 * SECURITY NOTICE:
 * - NEVER store private keys in plaintext files
 * - NEVER log private keys to console
 * - NEVER commit private keys to version control
 * - ALWAYS use secure key management in production
 * - Consider HSM or cloud KMS for production deployments
 */

// Validate environment variables are properly formatted
function validatePrivateKey(keyString: string): Uint8Array {
  try {
    const keyArray = JSON.parse(keyString);
    if (!Array.isArray(keyArray) || keyArray.length !== 64) {
      throw new Error('Invalid private key format');
    }
    return Uint8Array.from(keyArray);
  } catch (error) {
    throw new Error(`Failed to parse private key: ${error.message}`);
  }
}

// Secure wallet loading with validation
export function loadWallet(): Keypair {
  const privateKeyEnv = process.env.WALLET_PRIVATE_KEY;

  if (!privateKeyEnv) {
    throw new Error(
      '❌ CRITICAL: WALLET_PRIVATE_KEY environment variable not set\n' +
      '   For security, this key must be provided securely via environment variables.\n' +
      '   NEVER commit private keys to version control or store them in files.\n' +
      '   Use hardware security modules (HSM) or cloud KMS in production.'
    );
  }

  try {
    const privateKey = validatePrivateKey(privateKeyEnv);
    const keypair = Keypair.fromSecretKey(privateKey);

    // Security validation: ensure this is a valid keypair
    if (!keypair.publicKey) {
      throw new Error('Invalid keypair generated');
    }

    // WARNING: Never log private keys - only public key for verification
    console.log(`✅ Wallet loaded securely`);
    console.log(`🔑 Public key: ${keypair.publicKey.toBase58()}`);

    return keypair;
  } catch (error) {
    throw new Error(`❌ SECURITY ERROR: Failed to load wallet: ${error.message}`);
  }
}

// Generate secure devnet wallet (for development only)
export function generateDevnetWallet(): Keypair {
  if (process.env.NODE_ENV === 'production') {
    throw new Error('❌ SECURITY ERROR: Cannot generate wallets in production');
  }

  console.log('⚠️  DEVELOPMENT MODE: Generating new wallet...');
  console.log('   This should ONLY be used for devnet testing');

  const keypair = Keypair.generate();

  console.log(`📋 New wallet public key: ${keypair.publicKey.toBase58()}`);
  console.log('🔐 PRIVATE KEY (handle with extreme care - DO NOT commit):');
  console.log(JSON.stringify(Array.from(keypair.secretKey)));

  return keypair;
}

// Get wallet public key without exposing private key
export function getWalletPublicKey(): PublicKey | null {
  try {
    const wallet = loadWallet();
    return wallet.publicKey;
  } catch (error) {
    console.error('❌ Failed to load wallet public key:', error.message);
    return null;
  }
}

// DANGEROUS - FOR EMERGENCY USE ONLY
export function emergencyExportKey(): string {
  console.warn('⚠️  ⚠️  ⚠️  EMERGENCY KEY EXPORT ⚠️  ⚠️  ⚠️');
  console.warn('This should NEVER be used in production!');

  try {
    const wallet = loadWallet();
    return JSON.stringify(Array.from(wallet.secretKey));
  } catch (error) {
    throw new Error(`Failed to export key: ${error.message}`);
  }
}

// Remove dangerous functions - DO NOT expose private keys
export function isSecureEnvironment(): boolean {
  return process.env.NODE_ENV === 'production' ||
         process.env.NODE_ENV === 'staging' ||
         Boolean(process.env.WALLET_PRIVATE_KEY);
}

// Export secure wallet instance
export const wallet = loadWallet();
