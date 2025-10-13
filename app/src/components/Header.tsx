'use client';

import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import Link from 'next/link';

export default function Header() {
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
          <WalletMultiButton />
        </div>
      </div>
    </header>
  );
}
