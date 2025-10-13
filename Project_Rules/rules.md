# rules.md

Purpose
Enforce a tight 5‑day Solana devnet MVP for FlexiYield and ensure agents read governance files under Project_Rules/ before coding.

Always Read First (from Project_Rules/)
- prompt.md, agent.md, chat-titles.md, rules.md.
Policy: Agents must consult these files before planning, editing, or running tasks. If any is missing, halt and request it.

Golden Rules
- Devnet only. No mainnet, no real funds, no bridges.
- Single deposit asset (USDCd). Two holdings (USDCd, USDTd).
- One DEX path; if devnet DEX is flaky, use an internal swap simulator program.
- Mock oracles via PDAs/admin UI; no external APIs in critical paths.
- Manual “Rebalance Now” allowed. Keeper optional.
- Security: guardian pause; admin/multisig for config.

Do
- Use Anchor (Rust) for programs; emit events for deposits/rebalances/redemptions.
- Use SPL Token 2022 with 6 decimals to mimic stable UX.
- Provide scripts: airdrop SOL, mint USDCd/USDTd, deploy, seed demo state.
- Build deterministic UI reading on‑chain PDAs; clear error/loader states.
- Document repo structure, environment, commands, demo steps, and constraints.

Don’t
- Don’t integrate real money markets/yield farms.
- Don’t add cross‑chain or mainnet features.
- Don’t depend on external, flaky APIs for prices/APY.
- Don’t implement complex rebalancing; keep rules simple.
- Don’t expose admin keys client‑side.

Programs (MVP Requirements)
- Basket Program: initialize_basket, deposit_usdc, redeem_flex, update_config; PDAs for Vault/Config; FLEX mint authority; checks for signer/decimals/balances.
- Strategy Program: set_targets, set_thresholds, set_oracle_values (admin-gated); getters for UI.
- Rebalance Program: rebalance_once (deltas, swap, cap enforcement, state update, event); pause/unpause_rebalancing.

Frontend (Must‑Haves)
- Phantom devnet connect; faucet; mint USDCd.
- Deposit/Withdraw; show balances of USDCd/FLEX.
- Dashboard: NAV, composition chart, APY/peg badges, recent tx with explorer links.
- Admin (demo mode): targets sliders/inputs; APY/peg toggles; “Rebalance Now”.

Testing & Developer Experience
- Unit tests for deposit/redeem math, NAV calc, caps, and rebalance deltas.
- Seed script initializes devnet mints and PDAs.
- Optional localnet with single command for offline demo.

Performance & UX
- Show tx time and fee to highlight Solana speed/cost.
- Use optimistic UI then confirm with on‑chain state.
- Mobile‑friendly layout; avoid heavy assets.

Delivery Checklist
- /programs: basket, strategy, rebalance (Anchor).
- /app: Next/React client (TypeScript, pnpm).
- /scripts: deploy, airdrop, mint, seed, demo helpers.
- README: one‑command run, demo flow, constraints, roadmap.
- Fallback demo video/GIF.

Agent Compliance
- Before coding or planning: read Project_Rules/prompt.md, Project_Rules/agent.md, Project_Rules/chat-titles.md, Project_Rules/rules.md, and #project.md.
- If instructions conflict, rules.md > project.md > agent.md > prompt.md > chat-titles.md.
- Stop and request clarification if a requested change breaks these rules.

Post‑Hackathon Roadmap (for docs only)
- Add robust oracles, multi‑DEX routing, yield‑bearing stable integrations, audits, and governance.
