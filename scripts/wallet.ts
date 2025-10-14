import { Keypair, PublicKey } from '@solana/web3.js';
import * as crypto from 'crypto';
import * as fs from 'fs';

/**
 * 🔒 SECURE WALLET UTILITY - PRODUCTION READY
 *
 * SECURITY NOTICE:
 * - NEVER store private keys in plaintext files
 * - NEVER log private keys to console in production
 * - NEVER commit private keys to version control
 * - ALWAYS use secure key management in production
 * - Consider HSM or cloud KMS for production deployments
 *
 * SECRET KEY PRINTING:
 * - Set PRINT_SECRET_KEY=true to enable secret key output (development only)
 * - Use SAVE_SECRET_FILE=path to save secret key to file with restricted permissions
 * - Secret keys are hidden by default for security
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
    throw new Error(`Failed to parse private key: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

// Check if secret key printing is explicitly enabled
function isSecretPrintingEnabled(): boolean {
  return process.env.PRINT_SECRET_KEY === 'true';
}

// Get file path for saving secret key
function getSecretFilePath(): string | null {
  const filePath = process.env.SAVE_SECRET_FILE;
  return filePath || null;
}

// Securely handle secret key output
function handleSecretKeyOutput(keypair: Keypair): void {
  const secretKeyArray = Array.from(keypair.secretKey);
  const secretKeyString = JSON.stringify(secretKeyArray);
  
  if (isSecretPrintingEnabled()) {
    console.log('🔐 PRIVATE KEY (handle with extreme care - DO NOT commit):');
    console.log(secretKeyString);
  } else {
    console.log('🔐 Private key generated (hidden for security)');
    console.log('   Set PRINT_SECRET_KEY=true to display secret key');
  }

  const filePath = getSecretFilePath();
  if (filePath) {
    try {
      // Write with restrictive permissions (0o600 = owner read/write only)
      fs.writeFileSync(filePath, secretKeyString, { mode: 0o600 });
      console.log(`💾 Secret key saved to: ${filePath} (with restricted permissions)`);
    } catch (error) {
      console.error(`❌ Failed to save secret key to file: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
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
    throw new Error(`❌ SECURITY ERROR: Failed to load wallet: ${error instanceof Error ? error.message : 'Unknown error'}`);
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
  
  // Use secure secret key handling
  handleSecretKeyOutput(keypair);

  return keypair;
}

// Get wallet public key without exposing private key
export function getWalletPublicKey(): PublicKey | null {
  try {
    const wallet = loadWallet();
    return wallet.publicKey;
  } catch (error) {
    console.error('❌ Failed to load wallet public key:', error instanceof Error ? error.message : 'Unknown error');
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
    throw new Error(`Failed to export key: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

// Remove dangerous functions - DO NOT expose private keys
export function isSecureEnvironment(): boolean {
  return process.env.NODE_ENV === 'production' ||
         process.env.NODE_ENV === 'staging' ||
         Boolean(process.env.WALLET_PRIVATE_KEY);
}

// Print usage help for secret key options
export function printSecretKeyUsage(): void {
  console.log('🔐 SECRET KEY OPTIONS:');
  console.log('  Environment Variables:');
  console.log('    PRINT_SECRET_KEY=true    - Enable secret key printing to console (development only)');
  console.log('    SAVE_SECRET_FILE=<path>  - Save secret key to file with restricted permissions');
  console.log('');
  console.log('  Examples:');
  console.log('    PRINT_SECRET_KEY=true node wallet.ts');
  console.log('    SAVE_SECRET_FILE=./keypair.json node wallet.ts');
  console.log('    PRINT_SECRET_KEY=true SAVE_SECRET_FILE=./keypair.json node wallet.ts');
  console.log('');
  console.log('  ⚠️  WARNING: Only use secret key options in development environments!');
}

// Export secure wallet instance (lazy-loaded)
export function getWallet(): Keypair {
  return loadWallet();
}
