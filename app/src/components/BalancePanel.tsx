'use client';

import { useWallet } from '@solana/wallet-adapter-react';

export default function BalancePanel() {
  const { publicKey } = useWallet();

  if (!publicKey) {
    return (
      <div className="rounded-lg border border-gray-200 bg-white p-6">
        <h2 className="mb-4 text-lg font-semibold">Balances</h2>
        <p className="text-gray-500">Connect wallet to view balances</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-6">
      <h2 className="mb-4 text-lg font-semibold">Balances</h2>
      <div className="space-y-3">
        <div className="flex justify-between">
          <span className="text-gray-600">USDCd</span>
          <span className="font-mono">0.00</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">USDTd</span>
          <span className="font-mono">0.00</span>
        </div>
        <div className="flex justify-between border-t pt-3">
          <span className="font-semibold text-gray-800">FLEX</span>
          <span className="font-mono font-semibold">0.00</span>
        </div>
      </div>
    </div>
  );
}
