# FlexiYield Operational Scripts

This directory contains TypeScript scripts for setting up and managing the FlexiYield MVP on Solana devnet.

## Prerequisites

1. **Node.js and pnpm**: Ensure you have Node.js and pnpm installed
2. **Solana CLI**: Install the Solana CLI tool suite
3. **Devnet Configuration**: Configure Solana CLI for devnet

```bash
# Install pnpm if not already installed
npm install -g pnpm

# Configure Solana CLI for devnet
solana config set --url devnet

# Verify configuration
solana config get
```

## Available Scripts

### 1. Airdrop SOL (`airdrop-devnet-sol.ts`)

Airdrops SOL to configured keypairs on devnet.

**Usage:**
```bash
cd app
pnpm airdrop
```

**What it does:**
- Loads or generates admin, user, and guardian keypairs
- Airdrops 2 SOL to each keypair if balance is below 2 SOL
- Saves keypairs to `.keypairs.json` for reuse

**Output:**
- Keypairs saved to `.keypairs.json`
- Transaction signatures for each airdrop

### 2. Create Token Mints (`create-mints.ts`)

Creates SPL Token mints for USDCd, USDTd, and FLEX tokens.

**Usage:**
```bash
cd app
pnpm create-mints
```

**What it does:**
- Creates three SPL token mints with 6 decimals each:
  - USDCd: USDC Devnet token
  - USDTd: USDT Devnet token
  - FLEX: FlexiYield liquidity token
- Creates associated token accounts for the user
- Mints initial tokens to user accounts (except FLEX)
- Saves mint addresses to `.env.local`

**Output:**
- Mint addresses saved to `.env.local`
- Transaction signatures for mint creation and initial funding

### 3. Seed Balances (`seed-balances.ts`)

Seeds vault accounts and demo accounts with initial token balances.

**Usage:**
```bash
cd app
pnpm seed-balances
```

**What it does:**
- Creates and funds vault accounts for each token
- Seeds vaults with 10,000 tokens each (for demo purposes)
- Funds user accounts with additional 5,000 tokens each
- Saves vault addresses to `.env.local`

**Output:**
- Vault addresses saved to `.env.local`
- Transaction signatures for vault seeding

### 4. Deploy Programs (`deploy-programs.ts`)

Builds and deploys all Anchor programs to devnet.

**Usage:**
```bash
cd app
pnpm deploy-programs
```

**What it does:**
- Builds all Anchor programs (basket, strategy, rebalance)
- Deploys programs to devnet
- Generates and copies IDLs to `app/src/idl/`
- Saves program IDs to `.env.local`

**Output:**
- Program IDs saved to `.env.local`
- IDL files copied to `app/src/idl/`
- Transaction signatures for program deployments

### 5. Demo Helper (`demo-helper.ts`)

Automates demo scenarios and strategy simulations.

**Usage:**
```bash
cd app
pnpm demo
```

**What it does:**
- Runs 5 pre-configured demo scenarios:
  1. Balanced Portfolio (50/50 split)
  2. USDCd Heavy (80/20 split)
  3. USDTd Heavy (20/80 split)
  4. Conservative (60/40 split, tight thresholds)
  5. Aggressive (70/30 split, loose thresholds)
- Simulates strategy updates and rebalances
- Records demo actions for later review

**Output:**
- Mock transaction signatures for demo purposes
- Demo recordings saved to `demo-recording.json`

## Convenience Scripts

### Full Setup
```bash
cd app
pnpm setup-devnet
```
Runs all setup scripts in sequence: airdrop → create-mints → seed-balances → deploy-programs

### Full Demo
```bash
cd app
pnpm full-demo
```
Runs complete setup followed by demo scenarios.

## Environment Variables

The scripts create and maintain several environment variables in `.env.local`:

```bash
# Token Mint Addresses
NEXT_PUBLIC_USDCD_MINT=<usdcd_mint_address>
NEXT_PUBLIC_USDTD_MINT=<usdtd_mint_address>
NEXT_PUBLIC_FLEX_MINT=<flex_mint_address>

# Vault Addresses
NEXT_PUBLIC_USDCD_VAULT=<uscd_vault_address>
NEXT_PUBLIC_USDTD_VAULT=<usdtd_vault_address>
NEXT_PUBLIC_FLEX_VAULT=<flex_vault_address>

# Program IDs
NEXT_PUBLIC_BASKET_PROGRAM=<basket_program_id>
NEXT_PUBLIC_STRATEGY_PROGRAM=<strategy_program_id>
NEXT_PUBLIC_REBALANCE_PROGRAM=<rebalance_program_id>

# Solana RPC (optional)
NEXT_PUBLIC_SOLANA_RPC_URL=<devnet_rpc_url>
```

## File Structure

```
scripts/
├── README.md                 # This documentation
├── airdrop-devnet-sol.ts     # SOL airdrop utility
├── create-mints.ts          # Token mint creation
├── seed-balances.ts         # Balance seeding utility
├── deploy-programs.ts       # Program deployment
└── demo-helper.ts           # Demo automation

app/src/lib/
└── solana-adapter.ts        # Shared Solana utilities

.root/
├── .keypairs.json           # Generated keypairs
└── .env.local              # Environment configuration
```

## Troubleshooting

### Common Issues

1. **Insufficient SOL**: Ensure your devnet account has enough SOL for transaction fees
2. **Network Issues**: Check your internet connection and devnet status
3. **Keyphrase Errors**: Ensure `.keypairs.json` exists and is valid
4. **Missing Dependencies**: Run `pnpm install` in the `app/` directory

### Recovery Commands

```bash
# Reset everything (start fresh)
rm .keypairs.json .env.local
cd app && pnpm setup-devnet

# Just re-airdrop if you run out of SOL
cd app && pnpm airdrop

# Just re-deploy programs if you made code changes
cd app && pnpm deploy-programs
```

## Best Practices

1. **Run scripts in sequence**: Always run `setup-devnet` before individual scripts
2. **Monitor transaction signatures**: Use Solana explorer to verify transactions
3. **Save your keypairs**: Backup `.keypairs.json` securely
4. **Check balances**: Verify tokens and SOL are distributed correctly
5. **Test incrementally**: Run individual scripts to debug issues

## Explorer Links

Transaction signatures can be viewed on Solana explorer:
- Devnet: https://solscan.io/tx/{signature}?cluster=devnet
- Mainnet (future): https://solscan.io/tx/{signature}

## Next Steps

After running the setup scripts:

1. Start the frontend: `cd app && pnpm dev`
2. Connect your wallet (use the admin keypair from `.keypairs.json`)
3. Navigate to the dashboard to see the initialized vaults
4. Test deposit and withdraw flows
5. Use the admin panel to adjust strategy parameters
6. Run `pnpm demo` to see automated strategy demonstrations

## Security Notes

- These scripts are for **devnet only**
- Never use the generated keypairs on mainnet
- Never commit `.keypairs.json` or `.env.local` to version control
- The demo helper uses mock transactions for demonstration purposes
- Always verify transactions on Solscan before proceeding