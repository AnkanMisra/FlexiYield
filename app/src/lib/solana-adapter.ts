// Buffer polyfill for browser compatibility
import { Buffer } from 'buffer';
if (typeof window !== 'undefined') {
  window.Buffer = Buffer;
} else if (typeof globalThis !== 'undefined') {
  globalThis.Buffer = Buffer;
}

// Re-export from shared utilities
export {
  ConnectionManager,
  WalletManager,
  TransactionUtils,
  PDAUtils,
  FLEXIYIELD_CONSTANTS
} from '../../../shared/lib/solana-adapter';
