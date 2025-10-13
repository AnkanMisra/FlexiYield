'use client';

export default function CompositionChart() {
  return (
    <div className="rounded-lg border border-gray-200 bg-white p-6">
      <h2 className="mb-4 text-lg font-semibold">Portfolio Composition</h2>
      <div className="flex h-48 items-center justify-center text-gray-400">
        Chart placeholder - Will show USDCd/USDTd distribution
      </div>
      <div className="mt-4 space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-gray-600">USDCd</span>
          <span className="font-mono">50.0%</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-gray-600">USDTd</span>
          <span className="font-mono">50.0%</span>
        </div>
      </div>
    </div>
  );
}
