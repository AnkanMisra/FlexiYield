import { Keypair } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";
import { wallet } from "./wallet";
import fs from "fs";
import path from "path";

const DEVNET_RPC = process.env.DEVNET_RPC || "https://api.devnet.solana.com";
const TOKEN_DECIMALS = 6;

interface TokenAddresses {
  usdcMint: string;
  usdtMint: string;
  flexMint: string;
}

/**
 * Creates the three required token mints for the MVP:
 * - USDCd (devnet USD Coin)
 * - USDTd (devnet Tether)
 * - FLEX (basket receipt token)
 */
async function createMints(): Promise<TokenAddresses> {
  console.log("🪙 Creating token mints on devnet...");

  const connection = new Connection(DEVNET_RPC, "confirmed");

  try {
    // Check if mints already exist
    const addressesPath = path.join(__dirname, "addresses.json");
    if (fs.existsSync(addressesPath)) {
      const addresses = JSON.parse(
        fs.readFileSync(addressesPath, "utf8"),
      ) as TokenAddresses;
      console.log("📋 Found existing token addresses:");
      console.log(`USDCd: ${addresses.usdcMint}`);
      console.log(`USDTd: ${addresses.usdtMint}`);
      console.log(`FLEX: ${addresses.flexMint}`);

      // Verify mints exist on-chain
      try {
        await Promise.all([
          connection.getAccountInfo(new PublicKey(addresses.usdcMint)),
          connection.getAccountInfo(new PublicKey(addresses.usdtMint)),
          connection.getAccountInfo(new PublicKey(addresses.flexMint)),
        ]);
        console.log("✅ All existing mints verified on-chain");
        return addresses;
      } catch (error) {
        console.log("⚠️  Some mints not found on-chain, creating new ones...");
      }
    }

    // Create USDCd mint
    console.log("Creating USDCd mint...");
    const usdcMint = await createMint(
      connection,
      wallet, // payer
      wallet.publicKey, // mint authority
      wallet.publicKey, // freeze authority
      TOKEN_DECIMALS,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`✅ USDCd mint: ${usdcMint.toBase58()}`);

    // Create USDTd mint
    console.log("Creating USDTd mint...");
    const usdtMint = await createMint(
      connection,
      wallet,
      wallet.publicKey,
      wallet.publicKey,
      TOKEN_DECIMALS,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`✅ USDTd mint: ${usdtMint.toBase58()}`);

    // Create FLEX mint
    console.log("Creating FLEX mint...");
    const flexMint = await createMint(
      connection,
      wallet,
      wallet.publicKey, // Will be transferred to basket PDA after initialization
      wallet.publicKey,
      TOKEN_DECIMALS,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`✅ FLEX mint: ${flexMint.toBase58()}`);

    const addresses: TokenAddresses = {
      usdcMint: usdcMint.toBase58(),
      usdtMint: usdtMint.toBase58(),
      flexMint: flexMint.toBase58(),
    };

    // Save addresses to file
    fs.writeFileSync(addressesPath, JSON.stringify(addresses, null, 2));
    console.log(`💾 Addresses saved to ${addressesPath}`);

    // Update .env.local file
    const envPath = path.join(__dirname, "../app/.env.local");
    let envContent = "";

    if (fs.existsSync(envPath)) {
      envContent = fs.readFileSync(envPath, "utf8");
    }

    // Update or add mint addresses
    const envVars = [
      `MINT_USDC=${addresses.usdcMint}`,
      `MINT_USDT=${addresses.usdtMint}`,
      `MINT_FLEX=${addresses.flexMint}`,
    ];

    envVars.forEach((envVar) => {
      const [key] = envVar.split("=");
      const regex = new RegExp(`^${key}=.*$`, "m");
      if (envContent.match(regex)) {
        envContent = envContent.replace(regex, envVar);
      } else {
        envContent += (envContent.endsWith("\n") ? "" : "\n") + envVar + "\n";
      }
    });

    fs.writeFileSync(envPath, envContent);
    console.log(`💾 .env.local updated with mint addresses`);

    return addresses;
  } catch (error) {
    console.error("❌ Failed to create mints:", error);
    process.exit(1);
  }
}

// Execute if run directly
if (require.main === module) {
  createMints()
    .then(() => {
      console.log("✅ Token mints created successfully");
      process.exit(0);
    })
    .catch((error) => {
      console.error("❌ Mint creation failed:", error);
      process.exit(1);
    });
}

export { createMints };
