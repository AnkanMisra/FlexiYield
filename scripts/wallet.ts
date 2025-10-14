import { Keypair } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';

/**
 * Simple wallet utility for scripts
 * In production, this should use a proper key management system
 */

let walletInstance: Keypair | null = null;

export const wallet = (() => {
  if (walletInstance) {
    return walletInstance;
  }

  // Try to load from environment variable first
  if (process.env.PRIVATE_KEY) {
    const privateKey = JSON.parse(process.env.PRIVATE_KEY);
    walletInstance = Keypair.fromSecretKey(Uint8Array.from(privateKey));
    return walletInstance;
  }

  // Try to load from file
  const walletPath = path.join(__dirname, 'wallet-keypair.json');
  if (fs.existsSync(walletPath)) {
    const secretKey = JSON.parse(fs.readFileSync(walletPath, 'utf8'));
    walletInstance = Keypair.fromSecretKey(Uint8Array.from(secretKey));
    return walletInstance;
  }

  // Generate new wallet for demo purposes
  console.log('⚠️  No wallet found, generating new one...');
  walletInstance = Keypair.generate();

  // Save wallet for future use
  fs.writeFileSync(walletPath, JSON.stringify(Array.from(walletInstance.secretKey)));
  console.log(`💾 New wallet saved to ${walletPath}`);
  console.log(`📋 Wallet public key: ${walletInstance.publicKey.toBase58()}`);
  console.log('⚠️  Fund this wallet with SOL using: airdrop-devnet-sol.ts');

  return walletInstance;
})();

export function saveWalletToEnv(): void {
  const envPath = path.join(__dirname, '../app/.env.local');
  let envContent = '';

  if (fs.existsSync(envPath)) {
    envContent = fs.readFileSync(envPath, 'utf8');
  }

  const walletKey = `WALLET_PRIVATE_KEY=${JSON.stringify(Array.from(wallet.secretKey))}`;
  const walletPubkey = `WALLET_PUBKEY=${wallet.publicKey.toBase58()}`;

  // Update or add wallet keys
  [walletKey, walletPubkey].forEach(envVar => {
    const [key] = envVar.split('=');
    const regex = new RegExp(`^${key}=.*$`, 'm');
    if (envContent.match(regex)) {
      envContent = envContent.replace(regex, envVar);
    } else {
      envContent += (envContent.endsWith('\n') ? '' : '\n') + envVar + '\n';
    }
  });

  fs.writeFileSync(envPath, envContent);
  console.log(`💾 Wallet credentials saved to ${envPath}`);
}
