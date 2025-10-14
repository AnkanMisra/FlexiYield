'use client';

import dynamic from 'next/dynamic';
import Link from 'next/link';
import { useState, useEffect } from 'react';

// Dynamically import WalletMultiButton to avoid hydration mismatch
const WalletMultiButton = dynamic(
  () => import('@solana/wallet-adapter-react-ui').then(mod => ({ default: mod.WalletMultiButton })),
  {
    ssr: false,
    loading: () => (
      <div className="h-10 w-32 bg-gray-200 rounded-lg animate-pulse" />
    )
  }
);

export default function Header() {
  const [isMounted, setIsMounted] = useState(false);

  useEffect(() => {
    setIsMounted(true);
  }, []);

  return (
    <header className="border-b border-gray-200 bg-white shadow-sm">
      <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-4">
        <div className="flex items-center gap-8">
          <Link href="/" className="text-2xl font-bold text-blue-600">
            FlexiYield
          </Link>
          <nav className="flex gap-4">
            <Link href="/" className="text-gray-700 hover:text-blue-600">
              Dashboard
            </Link>
            <Link href="/admin" className="text-gray-700 hover:text-blue-600">
              Admin
            </Link>
          </nav>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-500">Devnet</span>
          {isMounted && <WalletMultiButton />}
        </div>
      </div>
    </header>
  );
}
