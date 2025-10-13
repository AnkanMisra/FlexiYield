System setup:
- You are "GitHub Copilot".
- Before any action, OPEN and READ these files:
  1) Project_Rules/rules.md
  2) Project_Rules/project.md
  3) Project_Rules/prompt.txt
  4) Project_Rules/agent.txt
  5) Project_Rules/chat-title.txt
   6) Project_Rules/AGENT_RULES_AND_GUIDE.md
- Treat rules.md as the highest priority, then project.md, then agent.txt, then prompt.txt, then chat-title.txt.

Project constraints:
- 5-day MVP on Solana devnet ONLY, no real funds, no mainnet.
- pnpm + Next.js + TypeScript frontend; Anchor (Rust) programs; SPL Token 2022; @coral-xyz/anchor client.
- Single deposit asset (USDCd), two holdings (USDCd, USDTd).
- Rebalance via one DEX path OR internal swap simulator if DEX is unreliable.
- Mock oracles/APY/peg via PDAs/admin UI; manual “Rebalance Now”.

Your tasks (execute sequentially, committing at logical checkpoints):
1) Scaffold repo structure if missing:
   - programs/basket, programs/strategy, programs/rebalance
   - app (Next.js + TypeScript)
   - scripts (airdrop, mints, seed, deploy, demo helpers)
   - app/src/idl (IDL output folder)
   - Add CONTRIBUTING.md pointer to Project_Rules/* and Project_Rules/AGENT_RULES_AND_GUIDE.md
2) Initialize Next.js app (TypeScript) in ./app using pnpm:
   - Add: @solana/wallet-adapter-react, @solana/wallet-adapter-wallets, @solana/wallet-adapter-react-ui, @solana/web3.js, @coral-xyz/anchor, @solana/spl-token
   - Create a minimal layout: wallet connect, balances panel placeholders, composition chart placeholder, admin panel route (/admin).
   - Add .env.local.sample with DEVNET_RPC, MINT_USDCd, MINT_USDTd, MINT_FLEX placeholders.
3) Initialize Anchor workspace at root:
   - Anchor.toml, program skeletons for basket/strategy/rebalance.
   - Shared accounts/types; PDAs for vault/config/strategy/oracle; events.
   - Build pipeline to export IDLs into app/src/idl on each build/deploy.
4) Scripts (TypeScript in /scripts):
   - airdrop-devnet-sol.ts
   - create-mints.ts (USDCd/USDTd/FLEX, 6 decimals) → write addresses to scripts/addresses.json and app/.env.local
   - seed-balances.ts (mint USDCd to a demo user)
   - deploy-programs.ts (anchor build/deploy + export IDLs)
5) Implement Basket program (MVP):
   - initialize_basket, deposit_usdc (mint FLEX 1:1 vs NAV), redeem_flex (return USDCd)
   - Events: Deposit, Redeem
   - NAV = sum(vault_balances) with 1.0 assumption per stable (MVP)
6) Implement Strategy + Rebalance:
   - Strategy: set_targets, set_thresholds, set_caps, set_oracle_values (mock APY/peg); getters
   - Rebalance: rebalance_once (delta calc, caps, single swap via DEX CPI if available or internal swap simulator), pause/unpause; Rebalanced event
7) Frontend wiring:
   - Wallet connect (Phantom devnet), faucet CTA, “Mint USDCd” CTA
   - Deposit/Withdraw flows; show NAV, FLEX supply, composition, APY/peg badges, tx history with explorer links
   - /admin: controls for targets/thresholds/caps, APY/peg toggles, Rebalance Now button
8) Minimal tests:
   - Deposit/redeem math, NAV, caps, delta computation
9) Documentation:
   - Update README.md with one-command setup/run, demo steps, and constraints
   - Ensure CONTRIBUTING.md and Project_Rules/AGENT_RULES_AND_GUIDE.md are present and point to Project_Rules/*

Important:
- Do NOT add mainnet, bridges, real money market integrations, or external APIs in critical paths.
- If DEX CPI fails on devnet, implement an internal swap simulator program (constant product, small fee) and clearly label it.

When ready, begin with tasks (1) and (2). Use TypeScript strict mode. Create small, focused commits. Provide any new file paths and how to run the local dev server (pnpm). Acknowledge after reading all governance files by listing the config values you will require (.env.local keys).
