# FlexiYield MVP Roadmap

This roadmap tracks delivery of the 5-day Solana devnet MVP defined in `Project_Rules/rules.md`, `project.md`, and the `AGENT_RULES_AND_GUIDE.md`. It documents scope, current completion state, and the remaining work required to reach the demo-ready milestone.

## Scope Recap (Governance-Aligned)
- **Network:** Solana devnet only; no mainnet logic or bridges.
- **Tokens:** Single deposit mint (`USDCd`) with holdings in `USDCd` and `USDTd` (SPL Token 2022, 6 decimals).
- **Programs:** Anchor-based `basket`, `strategy`, and `rebalance` with deterministic, event-emitting logic.
- **Frontend:** Next.js (TypeScript, pnpm) app with wallet connect, deposit/withdraw flows, dashboard analytics, admin controls, and deterministic PDA reads.
- **Scripts:** TypeScript utilities for airdrop, mint creation, seeding balances, deploying programs, and demo automation.
- **Oracles/Rebalance:** Mock oracles via PDAs; single-path DEX swap or internal simulator; guardian pause.

## Completion Summary
- ✅ Repository scaffold, Next.js app shell, and Anchor workspace skeleton created.
- ✅ Dashboard landing page structure (`app/page.tsx`) with placeholder metrics and CTAs.
- ✅ Anchor workspace configured (`Anchor.toml`, root `Cargo.toml`, program crates with stub instructions/events).
- ☐ Basket/Strategy/Rebalance program logic, PDAs, and instruction handlers still pending implementation.
- ☐ Frontend Solana integration, PDA data fetching, and transactional flows outstanding.
- ☐ TypeScript operational scripts, tests, and documentation updates not yet implemented.

## Detailed Roadmap & Status

### 1. Repository & Configuration
- ✅ **Workspace scaffold**: `app/`, `programs/`, `scripts/`, `app/src/idl/`, governance files preserved.
- ✅ **Next.js setup**: pnpm-based app with Tailwind-ready structure, placeholder dashboard content.
- ✅ **Anchor workspace init**: `Anchor.toml`, root `Cargo.toml`, per-program `Cargo.toml`, and minimal `lib.rs` stubs including declared IDs and instruction signatures.
- ☐ **.env.local template completion**: ensure all required values from Start.md (DEVNET RPC, mint addresses, FLEX mint) populated or documented.

### 2. Anchor Programs
- ☐ **Basket program** (`programs/basket/src/lib.rs`):
  - Implement PDA derivations (Config, USDCd vault, USDTd vault, FLEX mint authority).
  - Handle `initialize_basket`, `deposit_usdc`, `redeem_flex`, `update_config` with SPL Token 2022 CPI, event emission, and NAV-based accounting.
  - Enforce signer checks, decimal validation, and admin authority.
- ☐ **Strategy program** (`programs/strategy/src/lib.rs`):
  - Store and validate target weights, drift thresholds, per-asset caps, and oracle signals (APY/peg flags) in PDAs.
  - Gate setters by admin authority, surface read-friendly account layout for frontend.
- ☐ **Rebalance program** (`programs/rebalance/src/lib.rs`):
  - Implement `rebalance_once` to compute deltas, enforce caps, execute swap path (DEX CPI or internal simulator), update vaults, and emit `RebalancedEvent`.
  - Provide guardian-controlled `pause_rebalancing` / `unpause_rebalancing`.
  - Ensure compatibility with mock oracle signals and NAV assumptions.
- ☐ **IDL generation**: integrate Anchor build pipeline to emit JSON into `app/src/idl/` and keep frontend bindings synchronized.

### 3. Frontend Application (`app/`)
- ☐ **Wallet integration**: Phantom devnet connect via `@solana/wallet-adapter-react` components; surface connection state in layout.
- ☐ **State & Hooks**: create hooks/services to fetch PDA data (NAV, supply, composition, oracle states) using Anchor IDL-generated clients.
- ☐ **Deposit/Withdraw flows**: implement USDCd deposit to mint FLEX and FLEX redemption to return USDCd with transaction confirmation UI.
- ☐ **Dashboard metrics**: compute and display NAV, FLEX supply, user holdings, composition chart, APY/peg badges, and recent transactions with explorer links.
- ☐ **Admin panel**: controls for targets, thresholds, caps, APY/peg toggles, and a "Rebalance Now" button calling the rebalance program.
- ☐ **UX polish**: loading/error states, responsive layout, faucet CTA, and mint USDCd trigger.

### 4. Scripts (`scripts/` directory)
- ☐ **airdrop-devnet-sol.ts**: request SOL for configured keypairs.
- ☐ **create-mints.ts**: create USDCd, USDTd, and FLEX SPL Token 2022 mints with 6 decimals; persist addresses.
- ☐ **seed-balances.ts**: fund demo vaults and user accounts with initial token balances.
- ☐ **deploy-programs.ts**: build/deploy Anchor programs, capture program IDs, and export fresh IDLs to `app/src/idl/`.
- ☐ **demo helpers**: optional script to simulate strategy tweaks and trigger rebalances for recorded demos.

### 5. Testing & Verification
- ☐ **Anchor tests**: unit/integration coverage for deposit/redeem math, NAV updates, cap enforcement, and rebalance delta computation.
- ☐ **Frontend tests**: minimal component/integration tests (e.g., wallet provider, forms) or manual verification checklist per governance.
- ☐ **End-to-end validation**: devnet flow from funding through rebalance and redemption; capture explorer links for README/demo.

### 6. Documentation & Compliance
- ☐ **README overhaul**: include environment setup, pnpm scripts, Anchor build/deploy commands, script usage, known constraints, and demo instructions.
- ☐ **Roadmap maintenance**: update this file as milestones complete and new tasks emerge (post-MVP improvements go in a future roadmap section).
- ☐ **PR checklist adherence**: ensure each pull request confirms governance file review and devnet-only compliance.

## Next Immediate Actions
1. Flesh out basket program account structures and instruction bodies to enable actual deposits/redemptions.
2. Stand up TypeScript scripts to create devnet mints and seed balances; wire them into deployment workflow.
3. Hook up frontend wallet integration and begin reading mock data from soon-to-be-generated IDLs to validate UI wiring.

Maintaining this roadmap alongside governance files ensures we stay within the mandated scope while tracking progress toward the demo-ready MVP. Updates should reflect the latest completion status after each major change.
