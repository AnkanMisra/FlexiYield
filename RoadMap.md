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
- ✅ Deployment configurations (Vercel, Nixpacks) finalized and tested.
- ✅ Frontend deployed successfully with wallet adapter infrastructure in place.
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

## Work Plan: Day 2 Onwards (Starting Oct 14, 2025)

### Day 2: Core Program Implementation
**Priority: Basket Program Foundation**
1. **Basket Program PDAs & State** (`programs/basket/src/lib.rs`)
   - Define `BasketConfig` account structure (admin authority, NAV, FLEX supply, vaults)
   - Implement PDA derivations for Config, USDCd vault, USDTd vault, FLEX mint authority
   - Add proper account validation and constraints with `#[account(...)]` macros

2. **Basket Core Instructions**
   - `initialize_basket`: Create config PDA, initialize token vaults, set mint authorities
   - `deposit_usdc`: Accept USDCd, mint FLEX based on NAV, emit `DepositEvent`
   - `redeem_flex`: Burn FLEX, return USDCd proportionally, emit `RedeemEvent`
   - Add decimal validation (6 decimals) and arithmetic safety checks

3. **Testing Setup**
   - Write Anchor tests for initialize → deposit → redeem flow
   - Verify NAV calculations and event emissions

**Deliverables:** Working basket program with deposit/redeem logic testable on devnet

---

### Day 3: Strategy Program & Script Infrastructure
**Priority: Strategy Logic + Operational Scripts**

1. **Strategy Program** (`programs/strategy/src/lib.rs`)
   - Define `StrategyConfig` PDA with target weights, drift thresholds, asset caps
   - Implement `set_targets`, `set_thresholds`, `set_caps` with admin-only access
   - Add oracle signal fields (APY flags, peg status) with setters
   - Ensure read-friendly account layout for frontend queries

2. **TypeScript Scripts** (`scripts/`)
   - `airdrop-devnet-sol.ts`: Fund keypairs with devnet SOL
   - `create-mints.ts`: Create USDCd, USDTd, FLEX mints; write addresses to `.env`
   - `seed-balances.ts`: Fund demo accounts and vaults with initial tokens
   - `deploy-programs.ts`: Build/deploy all programs, capture IDs, copy IDLs to `app/src/idl/`
   - Wire scripts into package.json or document run sequence

3. **Environment Configuration**
   - Complete `.env.example` with all required variables
   - Document script execution order in README
   - Test full deploy → mint → seed pipeline

**Deliverables:** Strategy program deployed, scripts operational, devnet environment reproducible

---

### Day 4: Rebalance Program & Frontend Integration
**Priority: Rebalance Logic + Live Data Display**

1. **Rebalance Program** (`programs/rebalance/src/lib.rs`)
   - Implement `rebalance_once`: compute deltas from strategy targets vs current holdings
   - Add swap simulation or DEX CPI path (devnet Jupiter/Orca or mock transfer)
   - Enforce per-asset caps, update basket NAV and vault balances
   - Emit `RebalancedEvent` with before/after composition
   - Add guardian pause/unpause controls

2. **Frontend Data Fetching** (`app/src/`)
   - Create hooks to read BasketConfig, StrategyConfig PDAs using Anchor client
   - Display live NAV, FLEX supply, user wallet holdings
   - Show current composition (USDCd/USDTd percentages) with chart component
   - Surface oracle signals (APY/peg badges)

3. **Transaction Flows UI**
   - Deposit form: input USDCd amount, estimate FLEX output, submit transaction
   - Withdraw form: input FLEX amount, estimate USDCd return, submit transaction
   - Show transaction confirmation with Solana explorer links
   - Add loading states and error handling

**Deliverables:** Rebalance program functional, frontend displays live devnet data, deposit/redeem flows work end-to-end

---

### Day 5: Admin Panel, Testing & Documentation
**Priority: Admin Controls + Polish + Handoff**

1. **Admin Panel** (`app/app/admin/page.tsx` or modal)
   - Forms to update strategy targets, thresholds, caps
   - Toggle oracle signals (mock APY/peg indicators)
   - "Rebalance Now" button to trigger `rebalance_once`
   - Display recent rebalance events with composition changes
   - Protect routes/UI with admin wallet check

2. **Comprehensive Testing**
   - Anchor integration tests for all programs
   - Frontend: manual smoke test checklist (wallet connect, deposit, withdraw, rebalance)
   - End-to-end devnet flow: fund → deploy → seed → interact → verify on explorer
   - Capture screenshots/recording for demo

3. **Documentation & Cleanup**
   - Update README with complete setup instructions
   - Document environment variables, script usage, deployment steps
   - Add troubleshooting section for common devnet issues
   - Record demo video showing deposit → rebalance → withdraw flow
   - Final governance review: confirm devnet-only, no mainnet references

4. **Code Review & Refinement**
   - Review all error handling and edge cases
   - Ensure consistent logging and event emissions
   - Check decimal arithmetic for overflow safety
   - Validate admin authority checks across all programs

**Deliverables:** Fully functional MVP on devnet, admin controls operational, documentation complete, demo-ready

---

## Post-MVP Enhancements (Future Backlog)
- Real oracle integration (Pyth/Switchboard)
- Multi-hop DEX routing for optimal swaps
- Historical analytics and performance tracking
- Automated rebalancing scheduler (cron/clockwork)
- Enhanced UX: portfolio projections, gas estimates
- Mainnet preparation (after thorough audit)

---

## Daily Standup Questions
1. **What did we complete yesterday?**
2. **What are we working on today?**
3. **Any blockers or risks?**
4. **Are we on track for the 5-day MVP deadline?**

Maintaining this roadmap alongside governance files ensures we stay within the mandated scope while tracking progress toward the demo-ready MVP. Updates should reflect the latest completion status after each major change.
