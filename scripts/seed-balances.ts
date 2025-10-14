import { Keypair } from "@solana/web3.js";
import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";
import { wallet } from "./wallet";
import fs from "fs";
import path from "path";

const DEVNET_RPC = process.env.DEVNET_RPC || "https://api.devnet.solana.com";
const SEED_AMOUNT = 100_000_000; // 100 USDCd/USDTd each (6 decimals)

interface TokenAddresses {
  usdcMint: string;
  usdtMint: string;
  flexMint: string;
}

/**
 * Seeds token accounts for testing:
 * - Seeds demo user wallet with USDCd for testing deposits
 * - Seeds vaults with initial USDCd/USDTd balances
 */
async function seedBalances(): Promise<void> {
  console.log("🌱 Seeding token balances for testing...");

  const connection = new Connection(DEVNET_RPC, "confirmed");

  try {
    // Load token addresses
    const addressesPath = path.join(__dirname, "addresses.json");
    if (!fs.existsSync(addressesPath)) {
      console.error("❌ Token addresses not found. Run create-mints.ts first");
      process.exit(1);
    }

    const addresses = JSON.parse(
      fs.readFileSync(addressesPath, "utf8"),
    ) as TokenAddresses;
    const usdcMint = new PublicKey(addresses.usdcMint);
    const usdtMint = new PublicKey(addresses.usdtMint);

    console.log("📋 Using token addresses:");
    console.log(`USDCd: ${usdcMint.toBase58()}`);
    console.log(`USDTd: ${usdtMint.toBase58()}`);

    // Create user token accounts
    console.log("Creating user token accounts...");

    const userUsdcAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet,
      usdcMint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`✅ User USDCd account: ${userUsdcAccount.address.toBase58()}`);

    const userUsdtAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet,
      usdtMint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`✅ User USDTd account: ${userUsdtAccount.address.toBase58()}`);

    // Mint tokens to user account
    console.log(`Minting ${SEED_AMOUNT / 1_000_000} USDCd to user...`);
    await mintTo(
      connection,
      wallet,
      usdcMint,
      userUsdcAccount.address,
      wallet.publicKey,
      SEED_AMOUNT
    );

    console.log(`Minting ${SEED_AMOUNT / 1_000_000} USDTd to user...`);
    await mintTo(
      connection,
      wallet,
      usdtMint,
      userUsdtAccount.address,
      wallet.publicKey,
      SEED_AMOUNT
    );

    // Check final balances
    const finalUsdcBalance = await connection.getTokenAccountBalance(
      userUsdcAccount.address,
    );
    const finalUsdtBalance = await connection.getTokenAccountBalance(
      userUsdtAccount.address,
    );

    console.log("✅ Final user balances:");
    console.log(
      `USDCd: ${parseInt(finalUsdcBalance.value.amount) / 1_000_000}`,
    );
    console.log(
      `USDTd: ${parseInt(finalUsdtBalance.value.amount) / 1_000_000}`,
    );

    // Create a demo vault address for testing (this would be the actual vault PDA in production)
    const demoVault = Keypair.generate();
    console.log(`Demo vault address: ${demoVault.publicKey.toBase58()}`);

    // Save demo vault address for reference
    const demoPath = path.join(__dirname, "demo-vault.json");
    fs.writeFileSync(
      demoPath,
      JSON.stringify(
        {
          vault: demoVault.publicKey.toBase58(),
        },
        null,
        2,
      ),
    );

    console.log("✅ Token seeding completed successfully");
    console.log("📝 Demo wallet ready for testing deposits and rebalancing");
  } catch (error) {
    console.error("❌ Failed to seed balances:", error);
    process.exit(1);
  }
}

// Execute if run directly
if (require.main === module) {
  seedBalances()
    .then(() => {
      console.log("✅ Balance seeding completed");
      process.exit(0);
    })
    .catch((error) => {
      console.error("❌ Balance seeding failed:", error);
      process.exit(1);
    });
}

export { seedBalances };
