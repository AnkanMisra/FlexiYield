import { Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";

/**
 * Solana connection manager for FlexiYield
 * Handles devnet connection configuration and utilities
 */
export class ConnectionManager {
  private connection: Connection;

  constructor() {
    const rpcUrl =
      process.env.DEVNET_RPC ||
      process.env.NEXT_PUBLIC_SOLANA_RPC_URL ||
      clusterApiUrl("devnet");
    this.connection = new Connection(rpcUrl, "confirmed");
  }

  public getConnection(): Connection {
    return this.connection;
  }

  public async getAccountBalance(publicKey: PublicKey): Promise<number> {
    const balance = await this.connection.getBalance(publicKey);
    return balance / 1e9; // Convert lamports to SOL
  }

  public async getTokenBalance(tokenAccount: PublicKey): Promise<number> {
    try {
      const accountInfo = await this.connection.getAccountInfo(tokenAccount);
      if (!accountInfo) {
        throw new Error("Token account not found");
      }

      // Parse token account data (simplified for demo)
      // In production, use @solana/spl-token account parsing
      return 0;
    } catch (error) {
      console.error("Error fetching token balance:", error);
      return 0;
    }
  }

  public async getLatestBlockhash() {
    return await this.connection.getLatestBlockhash();
  }
}

/**
 * Wallet connection utilities
 */
export class WalletManager {
  private static instance: WalletManager;
  private _publicKey: PublicKey | null = null;
  private _connected: boolean = false;

  static getInstance(): WalletManager {
    if (!WalletManager.instance) {
      WalletManager.instance = new WalletManager();
    }
    return WalletManager.instance;
  }

  get publicKey(): PublicKey | null {
    return this._publicKey;
  }

  get connected(): boolean {
    return this._connected;
  }

  connect(publicKey: PublicKey): void {
    this._publicKey = publicKey;
    this._connected = true;
  }

  disconnect(): void {
    this._publicKey = null;
    this._connected = false;
  }
}

/**
 * Transaction utilities
 */
export class TransactionUtils {
  static async confirmTransaction(
    connection: Connection,
    signature: string,
    commitment: "confirmed" | "finalized" = "confirmed",
  ): Promise<boolean> {
    try {
      const confirmation = await connection.confirmTransaction(
        signature,
        commitment,
      );
      return !confirmation.value.err;
    } catch (error) {
      console.error("Error confirming transaction:", error);
      return false;
    }
  }

  static getExplorerUrl(
    signature: string,
    cluster: "devnet" | "testnet" | "mainnet-beta" = "devnet",
  ): string {
    const baseUrl = "https://solscan.io";
    const url = `${baseUrl}/tx/${signature}`;
    return cluster === "mainnet-beta" ? url : `${url}?cluster=${cluster}`;
  }

  static getTokenExplorerUrl(
    address: string,
    cluster: "devnet" | "testnet" | "mainnet-beta" = "devnet",
  ): string {
    const baseUrl = "https://solscan.io";
    const url = `${baseUrl}/token/${address}`;
    return cluster === "mainnet-beta" ? url : `${url}?cluster=${cluster}`;
  }
}

/**
 * PDA derivation utilities for FlexiYield programs
 */
export class PDAUtils {
  static getBasketConfigPDA(programId: PublicKey): [PublicKey, number] {
    const seed = "basket-config";
    return PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
  }

  static getBasketVaultPDA(
    programId: PublicKey,
    mint: PublicKey,
  ): [PublicKey, number] {
    const seed = "vault";
    return PublicKey.findProgramAddressSync(
      [Buffer.from(seed), mint.toBuffer()],
      programId,
    );
  }

  static getBasketMintAuthorityPDA(programId: PublicKey): [PublicKey, number] {
    const seed = "mint-authority";
    return PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
  }

  static getStrategyConfigPDA(programId: PublicKey): [PublicKey, number] {
    const seed = "strategy-config";
    return PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
  }

  static getRebalanceConfigPDA(programId: PublicKey): [PublicKey, number] {
    const seed = "rebalance-config";
    return PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
  }
}

/**
 * Constants for FlexiYield
 */
export const FLEXIYIELD_CONSTANTS = {
  DECIMALS: 6,
  BPS_DENOMINATOR: 10000,
  DEFAULT_TARGET_WEIGHTS: {
    USDC_WEIGHT_BPS: 5000,
    USDT_WEIGHT_BPS: 5000,
  },
  DEFAULT_DRIFT_THRESHOLD_BPS: 500, // 5%
  DEFAULT_WEIGHT_CAPS: {
    USDC_CAP_BPS: 8000, // 80%
    USDT_CAP_BPS: 8000, // 80%
  },
} as const;
