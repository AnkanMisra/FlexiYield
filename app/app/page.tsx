import BalancePanel from "../src/components/BalancePanel";
import CompositionChart from "../src/components/CompositionChart";

export default function Home() {
	return (
		<div className="mx-auto max-w-7xl px-4 py-8">
			<div className="mb-8">
				<h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
				<p className="mt-2 text-gray-600">
					FlexiYield Basket - Solana Devnet Demo
				</p>
			</div>
			<div className="mb-6 grid gap-6 md:grid-cols-3">
				<div className="rounded-lg border border-gray-200 bg-white p-6">
					<div className="text-sm text-gray-600">Total NAV</div>
					<div className="mt-2 text-3xl font-bold">$0.00</div>
				</div>
				<div className="rounded-lg border border-gray-200 bg-white p-6">
					<div className="text-sm text-gray-600">FLEX Supply</div>
					<div className="mt-2 text-3xl font-bold">0</div>
				</div>
				<div className="rounded-lg border border-gray-200 bg-white p-6">
					<div className="text-sm text-gray-600">Your FLEX</div>
					<div className="mt-2 text-3xl font-bold">0</div>
				</div>
			</div>
			<div className="grid gap-6 md:grid-cols-2">
				<BalancePanel />
				<CompositionChart />
			</div>
			<div className="mt-6 rounded-lg border border-blue-200 bg-blue-50 p-4">
				<h3 className="font-semibold text-blue-900">Getting Started</h3>
				<ol className="mt-2 list-inside list-decimal space-y-1 text-sm text-blue-800">
					<li>Connect your Phantom wallet (Devnet)</li>
					<li>Get devnet SOL from the faucet</li>
					<li>Mint test USDCd tokens</li>
					<li>Deposit USDCd to receive FLEX tokens</li>
				</ol>
				<div className="mt-4 flex gap-4">
					<a
						href="https://faucet.solana.com"
						target="_blank"
						rel="noopener noreferrer"
						className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
					>
						Get Devnet SOL
					</a>
					<button className="rounded-md border border-blue-600 px-4 py-2 text-sm font-medium text-blue-600 hover:bg-blue-50">
						Mint USDCd (Coming Soon)
					</button>
				</div>
			</div>
		</div>
	);
}
