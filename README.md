<h1 align="center">FlexiYield</h1>

<p align="center">
  <img width="330" height="301" alt="Screenshot 2025-10-13 at 19 54 05" src="https://github.com/user-attachments/assets/b0948993-7b98-439c-863c-8e4e5aa3f653" />
</p>

<p align="center">
  <b>Solana-native dynamic stablecoin basket for auto-optimized, diversified yield.</b>
</p>

---

<p align="center">
FlexiYield turns USDC/USDT into an auto-optimized, diversified portfolio.  
</p>

 <p align="center"> 
It provides rules-based rebalancing, de-peg protection, and real-time analytics — all within a seamless DeFi experience.
</p>

---

## Getting Started

1. **Install dependencies**
  ```bash
  cd app
  pnpm install
  ```

2. **Run the Next.js dev server**
  ```bash
  pnpm dev
  ```
  The UI is available at `http://localhost:3000`.

3. **Build Anchor programs (optional for now)**
  ```bash
  cd ..
  anchor build
  ```

> Configure `.env.local` inside `app/` with the required `DEVNET_RPC`, `MINT_USDCd`, `MINT_USDTd`, and `MINT_FLEX` values before interacting with Solana.
