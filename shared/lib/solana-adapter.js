"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.FLEXIYIELD_CONSTANTS = exports.PDAUtils = exports.TransactionUtils = exports.WalletManager = exports.ConnectionManager = void 0;
const web3_js_1 = require("@solana/web3.js");
/**
 * Solana connection manager for FlexiYield
 * Handles devnet connection configuration and utilities
 */
class ConnectionManager {
    constructor() {
        const rpcUrl = process.env.DEVNET_RPC ||
            process.env.NEXT_PUBLIC_SOLANA_RPC_URL ||
            (0, web3_js_1.clusterApiUrl)("devnet");
        this.connection = new web3_js_1.Connection(rpcUrl, "confirmed");
    }
    getConnection() {
        return this.connection;
    }
    async getAccountBalance(publicKey) {
        const balance = await this.connection.getBalance(publicKey);
        return balance / 1e9; // Convert lamports to SOL
    }
    async getTokenBalance(tokenAccount) {
        try {
            const accountInfo = await this.connection.getAccountInfo(tokenAccount);
            if (!accountInfo) {
                throw new Error("Token account not found");
            }
            // Parse token account data (simplified for demo)
            // In production, use @solana/spl-token account parsing
            return 0;
        }
        catch (error) {
            console.error("Error fetching token balance:", error);
            return 0;
        }
    }
    async getLatestBlockhash() {
        return await this.connection.getLatestBlockhash();
    }
}
exports.ConnectionManager = ConnectionManager;
/**
 * Wallet connection utilities
 */
class WalletManager {
    constructor() {
        this._publicKey = null;
        this._connected = false;
    }
    static getInstance() {
        if (!WalletManager.instance) {
            WalletManager.instance = new WalletManager();
        }
        return WalletManager.instance;
    }
    get publicKey() {
        return this._publicKey;
    }
    get connected() {
        return this._connected;
    }
    connect(publicKey) {
        this._publicKey = publicKey;
        this._connected = true;
    }
    disconnect() {
        this._publicKey = null;
        this._connected = false;
    }
}
exports.WalletManager = WalletManager;
/**
 * Transaction utilities
 */
class TransactionUtils {
    static async confirmTransaction(connection, signature, commitment = "confirmed") {
        try {
            const confirmation = await connection.confirmTransaction(signature, commitment);
            return !confirmation.value.err;
        }
        catch (error) {
            console.error("Error confirming transaction:", error);
            return false;
        }
    }
    static getExplorerUrl(signature, cluster = "devnet") {
        const baseUrl = "https://solscan.io";
        const url = `${baseUrl}/tx/${signature}`;
        return cluster === "mainnet-beta" ? url : `${url}?cluster=${cluster}`;
    }
    static getTokenExplorerUrl(address, cluster = "devnet") {
        const baseUrl = "https://solscan.io";
        const url = `${baseUrl}/token/${address}`;
        return cluster === "mainnet-beta" ? url : `${url}?cluster=${cluster}`;
    }
}
exports.TransactionUtils = TransactionUtils;
/**
 * PDA derivation utilities for FlexiYield programs
 */
class PDAUtils {
    static getBasketConfigPDA(programId) {
        const seed = "basket-config";
        return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
    }
    static getBasketVaultPDA(programId, mint) {
        const seed = "vault";
        return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seed), mint.toBuffer()], programId);
    }
    static getBasketMintAuthorityPDA(programId) {
        const seed = "mint-authority";
        return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
    }
    static getStrategyConfigPDA(programId) {
        const seed = "strategy-config";
        return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
    }
    static getRebalanceConfigPDA(programId) {
        const seed = "rebalance-config";
        return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
    }
}
exports.PDAUtils = PDAUtils;
/**
 * Constants for FlexiYield
 */
exports.FLEXIYIELD_CONSTANTS = {
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
};
