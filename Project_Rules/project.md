# FlexiYield — Solana-Native Dynamic Stablecoin Basket (Devnet MVP, 5 Days)

Tooling & Stack
- Frontend: Next.js (TypeScript), pnpm, React, Tailwind/MUI, Solana wallet-adapter.
- Programs: Anchor (Rust), SPL Token 2022.
- Client: @coral-xyz/anchor, @solana/web3.js, @solana/spl-token, a simple chart lib.
- Network: Solana devnet only. No mainnet, no real funds.

Governance Files (READ FIRST, from Project_Rules/)
- Project_Rules/rules.md — Scope, constraints, do/don’t.
- Project_Rules/prompt.md — Copilot identity and tool-use directives.
- Project_Rules/agent.md — Programming assistant behavior in VS Code.
- Project_Rules/chat-titles.md — Chat title style.
Agent requirement: Before planning or editing, consult all files in Project_Rules/. Treat them as the single source of truth for scope and behavior.

Problem & Value
- Users holding stables must manually rotate for yield and face de‑peg risk.
- FlexiYield is a “meta‑stablecoin” basket (FLEX) auto-allocating between two devnet stables with rules-based targets, peg-health checks, and caps, rebalancing via a single DEX path (or internal swap simulator). Solana’s speed/fees enable frequent, transparent rebalances.

5‑Day Devnet MVP Scope
- Single deposit asset: USDCd (custom devnet mint).
- Holdings: USDCd + USDTd (custom devnet mints).
- Rebalancing: rules-based, one DEX integration (Raydium/Orca devnet) OR internal swap simulator if devnet liquidity is unreliable.
- Strategy params: target weights (e.g., 70/30), drift threshold (e.g., 5%), per-asset caps (e.g., 80%).
- Oracles: mock APY + peg flags via PDA/admin UI.
- Manual “Rebalance Now” (keeper optional).

High-Level Architecture
On‑chain (Anchor/Rust)
1) Basket Program
   - initialize_basket, deposit_usdc (mint FLEX 1:1 vs NAV), redeem_flex (burn FLEX → return USDCd), update_config.
   - PDAs: Vault (USDCd/USDTd token accounts), Config, FLEX mint authority.
   - Events: Deposit, Redeem, ConfigUpdated.

2) Strategy Program
   - set_targets, set_thresholds, set_oracle_values (admin-gated).
   - Stores: target weights, caps, drift threshold; mock oracle (APY, peg flags).

3) Rebalance Program
   - rebalance_once: compute deltas vs targets; execute DEX swap or internal simulator; enforce caps; update vault; emit Rebalanced.
   - pause/unpause_rebalancing (guardian).

Off‑chain (optional)
- Keeper script (Node/TS) to detect drift and call rebalance_once. For demo, manual button is enough.

Frontend (Next.js, TypeScript, pnpm)
- Wallet connect (Phantom devnet), faucet CTA (airdrop SOL), one‑click mint of USDCd.
- Deposit/Withdraw flows.
- Dashboard: composition pie, NAV, FLEX supply, estimated APY (mock), peg‑health badges, recent tx with explorer links.
- Admin (demo mode): update targets/thresholds; toggle APY/peg; “Rebalance Now”.

Data & Math (MVP simplifications)
- NAV in USDC terms; fixed 1.0 price per stable for demo; mock APY displayed as a signal.
- Trigger: |current_weight_i − target_weight_i| > drift_threshold OR APY preference.
- Caps: max_weight_i ≤ cap_i; peg caution reduces cap_i temporarily.

Security (MVP)
- Guardian pause on rebalancing; admin/multisig for config.
- Devnet only; unaudited; no real funds.

Deliverables
- Live devnet dApp URL, repo with /programs and /app, deploy scripts, seeded demo state, README with one‑command run and demo script, fallback recording/GIF.

Demo Script (judge‑friendly)
- Airdrop SOL + Mint USDCd → Deposit 25 USDCd → receive 25 FLEX.
- Toggle APY (mock) → drift shows → Rebalance Now → composition updates with tx link.
- Toggle peg caution → cap reduces allocation to that asset.
- Redeem 5 FLEX → receive ~5 USDCd.
