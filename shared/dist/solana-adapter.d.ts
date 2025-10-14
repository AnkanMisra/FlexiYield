import { Connection, PublicKey } from "@solana/web3.js";
/**
 * Solana connection manager for FlexiYield
 * Handles devnet connection configuration and utilities
 */
export declare class ConnectionManager {
    private connection;
    constructor();
    getConnection(): Connection;
    getAccountBalance(publicKey: PublicKey): Promise<number>;
    getTokenBalance(tokenAccount: PublicKey): Promise<number>;
    getLatestBlockhash(): Promise<Readonly<{
        blockhash: import("@solana/web3.js").Blockhash;
        lastValidBlockHeight: number;
    }>>;
}
/**
 * Wallet connection utilities
 */
export declare class WalletManager {
    private static instance;
    private _publicKey;
    private _connected;
    static getInstance(): WalletManager;
    get publicKey(): PublicKey | null;
    get connected(): boolean;
    connect(publicKey: PublicKey): void;
    disconnect(): void;
}
/**
 * Transaction utilities
 */
export declare class TransactionUtils {
    static confirmTransaction(connection: Connection, signature: string, commitment?: "confirmed" | "finalized"): Promise<boolean>;
    static getExplorerUrl(signature: string, cluster?: "devnet" | "testnet" | "mainnet-beta"): string;
    static getTokenExplorerUrl(address: string, cluster?: "devnet" | "testnet" | "mainnet-beta"): string;
}
/**
 * PDA derivation utilities for FlexiYield programs
 */
export declare class PDAUtils {
    static getBasketConfigPDA(programId: PublicKey): [PublicKey, number];
    static getBasketVaultPDA(programId: PublicKey, mint: PublicKey): [PublicKey, number];
    static getBasketMintAuthorityPDA(programId: PublicKey): [PublicKey, number];
    static getStrategyConfigPDA(programId: PublicKey): [PublicKey, number];
    static getRebalanceConfigPDA(programId: PublicKey): [PublicKey, number];
}
/**
 * Constants for FlexiYield
 */
export declare const FLEXIYIELD_CONSTANTS: {
    readonly DECIMALS: 6;
    readonly BPS_DENOMINATOR: 10000;
    readonly DEFAULT_TARGET_WEIGHTS: {
        readonly USDC_WEIGHT_BPS: 5000;
        readonly USDT_WEIGHT_BPS: 5000;
    };
    readonly DEFAULT_DRIFT_THRESHOLD_BPS: 500;
    readonly DEFAULT_WEIGHT_CAPS: {
        readonly USDC_CAP_BPS: 8000;
        readonly USDT_CAP_BPS: 8000;
    };
};
