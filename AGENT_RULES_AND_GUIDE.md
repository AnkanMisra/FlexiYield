# AGENT_RULES_AND_GUIDE

Purpose
This document defines mandatory rules and a practical guide for any automated coding agent (e.g., GitHub Copilot) contributing to the FlexiYield repository. The goal is to keep the 5‑day, Solana devnet MVP focused, safe, and demo‑ready.

Mandatory Pre‑Read (before any planning or edits)
Agents MUST open and read all of the following files in this order:
1) Project_Rules/rules.md — Source of truth for scope, do/don’t, priorities.
2) Project_Rules/project.md — Project vision, architecture, MVP scope (5 days).
3) Project_Rules/prompt.txt — Agent identity and tool‑use directives.
4) Project_Rules/agent.txt — Programming assistant behavior in VS Code.
5) Project_Rules/chat-title.txt — Chat title guidelines.

If any file is missing, STOP and request it. Do not proceed.

Scope Summary (do not override)
- Network: Solana devnet ONLY. No mainnet. No real funds. No bridges.
- Deposit Asset: Single deposit (USDCd, custom devnet mint).
- Holdings: Two tokens (USDCd, USDTd).
- Rebalancing: Rules‑based, single DEX path; if devnet DEX is unreliable, use an internal swap simulator program.
- Oracles: Mock APY + peg flags via PDAs/admin UI; no external APIs in critical paths.
- Security (MVP): Guardian pause; admin/multisig for config.
- Timeframe: 5‑day MVP optimized for a clean demo.

Tech Stack
- Frontend: Next.js (TypeScript), pnpm, React, Tailwind or MUI, Solana wallet‑adapter.
- Programs: Anchor (Rust), SPL Token 2022.
- Client: @coral-xyz/anchor, @solana/web3.js, @solana/spl-token.
- Charts: Recharts/ECharts (keep lightweight).

Repository Structure (expected)
- Project_Rules/ — Governance and agent instruction files (read first).
- programs/ — Anchor programs: basket, strategy, rebalance.
- app/ — Next.js + TypeScript frontend (pnpm).
- scripts/ — Devnet setup: airdrop, mint, seed, deploy, demo helpers.
- app/src/idl/ — Generated Anchor IDLs for client bindings.

Rules of Engagement (Agent)
- Always read Project_Rules/* and this file before coding or planning.
- Respect instruction precedence on conflicts:
  rules.md > project.md > agent.txt > prompt.txt > chat-title.txt
- Do not expand scope or add features outside the MVP without an approved issue.
- Prefer simplicity and determinism over novelty; the demo must be robust on devnet.

Prohibited (MVP)
- Mainnet logic, bridges, cross‑chain features.
- Real money markets/yield farms integrations.
- External APIs for critical paths (price/APY/peg) — mock them via PDAs/admin UI.
- Exposing admin keys in client code or committing secrets.
- Complex, opaque rebalancing logic; keep the rules clear and traceable.

Required Features (MVP)
On‑chain (Anchor/Rust)
- Basket Program:
  - initialize_basket, deposit_usdc (mint FLEX 1:1 vs NAV), redeem_flex (return USDCd), update_config.
  - PDAs: Vault (USDCd/USDTd token accounts), Config, FLEX mint authority.
  - Events: Deposit, Redeem.
- Strategy Program:
  - set_targets, set_thresholds, set_caps, set_oracle_values (admin‑gated).
  - Stores target weights, drift threshold, per‑asset caps, mock APY/peg flags.
- Rebalance Program:
  - rebalance_once: compute deltas, enforce caps, perform single‑path swap (DEX CPI or internal simulator), update vault, emit Rebalanced.
  - pause/unpause_rebalancing (guardian).

Frontend (Next.js, TypeScript, pnpm)
- Wallet connect (Phantom devnet), faucet link/CTA, one‑click mint USDCd.
- Deposit/Withdraw flows for USDCd and FLEX.
- Dashboard: NAV, FLEX supply, composition chart, APY/peg badges (from Strategy/Oracle PDAs), recent tx with explorer links.
- Admin (demo mode): sliders for targets, inputs for thresholds/caps, toggles for APY/peg, “Rebalance Now” button.

Scripts (TypeScript)
- airdrop-devnet-sol.ts, create-mints.ts (USDCd/USDTd/FLEX, 6 decimals), seed-balances.ts, deploy-programs.ts (build/deploy + export IDLs), demo helpers.

Math & Data (MVP simplifications)
- NAV in USDC terms; assume both stables ~1.0 for demo.
- Trigger rebalance when |current_weight_i − target_weight_i| > drift_threshold OR mock APY preference flips.
- Caps: max_weight_i ≤ cap_i; peg caution reduces cap_i temporarily.
- Redemption standardizes to USDCd for clean UX (FLEX burn → USDCd).

Agent Workflow (step‑by‑step)
1) Read governance: Project_Rules/* and this file.
2) Plan minimal edits. Use clear, small PRs. State which governance rules you’re following.
3) Implement programs using Anchor; emit events. Keep SPL decimals at 6.
4) Generate and export IDLs to app/src/idl; keep bindings in sync.
5) Implement frontend with deterministic reads from PDAs. Avoid hidden state.
6) Provide scripts to airdrop, mint USDCd/USDTd, seed, deploy.
7) Verify devnet demo flow end‑to‑end. Include explorer links.
8) Document run commands and demo script in README; do not alter scope.

PR Checklist (Agent must include)
- [ ] I read Project_Rules/rules.md and Project_Rules/project.md.
- [ ] Changes comply with devnet‑only, mock oracles, single‑DEX/simulator rules.
- [ ] No mainnet, bridges, external critical APIs, or scope creep.
- [ ] Added/updated scripts (airdrop/mint/seed/deploy) if needed.
- [ ] Basic tests pass; demo flow works on devnet; screenshots/GIF attached.

Demo Expectations
- Airdrop SOL + Mint USDCd.
- Deposit USDCd → receive FLEX.
- Toggle APY/peg flags (admin UI) → show drift/caps.
- Rebalance Now → show composition update + explorer link.
- Redeem FLEX → receive USDCd.
- UI surfaces tx time and fee to illustrate Solana speed/cost.

Error Handling & Fallbacks
- If devnet DEX CPI is flaky, switch to internal swap simulator program and note it in PR/README.
- If RPC is unstable, provide a recorded fallback demo or GIF.
- Show clear UI states: loading, error, and on‑chain confirmations.

Testing & Local Runs
- Minimal unit tests: deposit/redeem math, NAV calc, caps, delta computation for rebalance.
- Optional localnet: single command to bring up validator + programs + seed.

Security Posture (MVP)
- Guardian pause on rebalancing; admin/multisig for config updates.
- Store sensitive keys server‑side or in .env (never commit).
- Clear disclaimer in README: unaudited, devnet‑only, not for real funds.

Notes to Future Maintainers
This document governs agent behavior during the 5‑day MVP window. Any scope changes must be proposed via issue and approved. After MVP, revisit with audits, robust oracles, multi‑DEX routing, yield‑bearing integrations, and governance.

Acknowledgment
By operating as an automated agent in this repository, you agree to follow AGENT_RULES_AND_GUIDE and Project_Rules/* without deviation. If instructions conflict or context is missing, stop and request clarification.

