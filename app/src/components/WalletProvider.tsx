'use client';

import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { ConnectionProvider, WalletProvider as SolanaWalletProvider } from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';
import { useMemo } from 'react';

export default function WalletProvider({ children }: { children: React.ReactNode }) {
  const network = WalletAdapterNetwork.Devnet;
  const endpoint = useMemo(() => {
    const rpc = process.env.NEXT_PUBLIC_DEVNET_RPC;
    if (!rpc) {
      console.warn('[WalletProvider] Missing NEXT_PUBLIC_DEVNET_RPC; falling back to clusterApiUrl(Devnet).');
      if (process.env.NODE_ENV === 'production') {
        console.warn(
          '[WalletProvider] Production build running without dedicated RPC endpoint; configure NEXT_PUBLIC_DEVNET_RPC to avoid shared cluster limits.'
        );
      }
    }
    return rpc ?? clusterApiUrl(network);
  }, [network]);

  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <SolanaWalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>{children}</WalletModalProvider>
      </SolanaWalletProvider>
    </ConnectionProvider>
  );
}
