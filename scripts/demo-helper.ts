import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { ConnectionManager, PDAUtils } from '../shared/lib/solana-adapter';
import fs from 'fs';
import path from 'path';

/**
 * Demo helper script for FlexiYield
 * Simulates strategy tweaks and triggers rebalances for recorded demos
 * Usage: npx ts-node scripts/demo-helper.ts
 */
async function main() {
  console.log('🎬 FlexiYield Demo Helper');
  console.log('=========================');

  const connectionManager = new ConnectionManager();
  const connection = connectionManager.getConnection();

  // Load configuration
  const keypairs = loadKeypairs();
  const programIds = loadProgramIds();
  const mintAddresses = loadMintAddresses();

  const admin = keypairs.admin;
  const user = keypairs.user;

  console.log(`Admin: ${admin.publicKey.toBase58()}`);
  console.log(`User: ${user.publicKey.toBase58()}`);

  // Demo scenarios
  const scenarios = [
    {
      name: 'Balanced Portfolio',
      description: '50/50 USDCd/USDTd split',
      targets: { usdc_weight_bps: 5000, usdt_weight_bps: 5000 }
    },
    {
      name: 'USDCd Heavy',
      description: '80% USDCd, 20% USDTd',
      targets: { usdc_weight_bps: 8000, usdt_weight_bps: 2000 }
    },
    {
      name: 'USDTd Heavy',
      description: '20% USDCd, 80% USDTd',
      targets: { usdc_weight_bps: 2000, usdt_weight_bps: 8000 }
    },
    {
      name: 'Conservative',
      description: '60/40 split with tight thresholds',
      targets: { usdc_weight_bps: 6000, usdt_weight_bps: 4000 },
      threshold: { bps: 200 } // 2%
    },
    {
      name: 'Aggressive',
      description: '70/30 split with loose thresholds',
      targets: { usdc_weight_bps: 7000, usdt_weight_bps: 3000 },
      threshold: { bps: 1000 } // 10%
    }
  ];

  console.log('\n📊 Available Demo Scenarios:');
  scenarios.forEach((scenario, index) => {
    console.log(`${index + 1}. ${scenario.name} - ${scenario.description}`);
  });

  console.log('\n🚀 Running demo scenarios...');

  for (const scenario of scenarios) {
    console.log(`\n--- ${scenario.name} ---`);
    console.log(scenario.description);

    await runScenario(scenario, {
      connection,
      admin,
      programIds,
      mintAddresses
    });

    // Wait for user to observe changes
    console.log('⏸️  Pausing for observation...');
    await sleep(3000); // 3 second pause
  }

  console.log('\n🎉 Demo completed!');
  console.log('Check the Solana explorer for transaction details');
}

async function runScenario(
  scenario: any,
  context: {
    connection: Connection;
    admin: Keypair;
    programIds: Record<string, string>;
    mintAddresses: Record<string, string>;
  }
) {
  try {
    // 1. Update strategy targets
    console.log('📈 Updating strategy targets...');

    if (context.programIds.strategy) {
      // In a real implementation, you would call the strategy program
      console.log(`  USDCd weight: ${scenario.targets.usdc_weight_bps / 100}%`);
      console.log(`  USDTd weight: ${scenario.targets.usdt_weight_bps / 100}%`);

      // Mock transaction signature for demo purposes
      const mockSignature = generateMockSignature();
      console.log(`  ✅ Targets updated (mock: ${mockSignature})`);
    }

    // 2. Update threshold if specified
    if (scenario.threshold) {
      console.log('🎯 Updating drift threshold...');
      console.log(`  New threshold: ${scenario.threshold.bps / 100}%`);

      const mockSignature = generateMockSignature();
      console.log(`  ✅ Threshold updated (mock: ${mockSignature})`);
    }

    // 3. Simulate rebalance
    console.log('⚖️  Triggering rebalance...');

    const mockRebalanceSignature = generateMockSignature();
    console.log(`  ✅ Rebalance triggered (mock: ${mockRebalanceSignature})`);

    // 4. Display expected outcome
    console.log('📊 Expected portfolio changes:');
    console.log(`  Target: ${scenario.targets.usdc_weight_bps / 100}% USDCd / ${scenario.targets.usdt_weight_bps / 100}% USDTd`);
    console.log(`  Threshold: ${scenario.threshold ? scenario.threshold.bps / 100 : 5}%`);

  } catch (error) {
    console.error('❌ Scenario failed:', error);
  }
}

function loadKeypairs(): Record<string, Keypair> {
  const keypairsFile = path.join(process.cwd(), '.keypairs.json');

  if (!fs.existsSync(keypairsFile)) {
    throw new Error('Keypairs file not found. Run airdrop-devnet-sol.ts first.');
  }

  const keypairsData = JSON.parse(fs.readFileSync(keypairsFile, 'utf8'));
  const keypairs: Record<string, Keypair> = {};

  for (const [name, secretKey] of Object.entries(keypairsData)) {
    keypairs[name] = Keypair.fromSecretKey(new Uint8Array(secretKey as number[]));
  }

  return keypairs;
}

function loadProgramIds(): Record<string, string> {
  const envFile = path.join(process.cwd(), '.env.local');

  if (!fs.existsSync(envFile)) {
    throw new Error('.env.local file not found. Run deploy-programs.ts first.');
  }

  const envContent = fs.readFileSync(envFile, 'utf8');
  const env: Record<string, string> = {};

  envContent.split('\n').forEach(line => {
    const [key, value] = line.split('=');
    if (key && value) {
      env[key] = value;
    }
  });

  return {
    basket: env.NEXT_PUBLIC_BASKET_PROGRAM || '',
    strategy: env.NEXT_PUBLIC_STRATEGY_PROGRAM || '',
    rebalance: env.NEXT_PUBLIC_REBALANCE_PROGRAM || '',
  };
}

function loadMintAddresses(): Record<string, string> {
  const envFile = path.join(process.cwd(), '.env.local');

  if (!fs.existsSync(envFile)) {
    throw new Error('.env.local file not found. Run create-mints.ts first.');
  }

  const envContent = fs.readFileSync(envFile, 'utf8');
  const env: Record<string, string> = {};

  envContent.split('\n').forEach(line => {
    const [key, value] = line.split('=');
    if (key && value) {
      env[key] = value;
    }
  });

  return {
    USDCd: env.NEXT_PUBLIC_USDCD_MINT || '',
    USDTd: env.NEXT_PUBLIC_USDTD_MINT || '',
    FLEX: env.NEXT_PUBLIC_FLEX_MINT || '',
  };
}

function generateMockSignature(): string {
  // Generate a realistic-looking mock signature
  const chars = 'ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz123456789';
  let signature = '';
  for (let i = 0; i < 88; i++) {
    signature += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return signature;
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Additional demo utilities

export class DemoRecorder {
  private static recordings: any[] = [];

  static recordAction(action: string, data: any) {
    this.recordings.push({
      timestamp: new Date().toISOString(),
      action,
      data
    });
  }

  static exportRecordings(): string {
    return JSON.stringify(this.recordings, null, 2);
  }

  static saveRecordings(filename: string = 'demo-recording.json') {
    const filePath = path.join(process.cwd(), filename);
    fs.writeFileSync(filePath, this.exportRecordings());
    console.log(`Demo recording saved to ${filePath}`);
  }
}

export class DemoMetrics {
  static calculateAPY(weights: { usdc_weight_bps: number; usdt_weight_bps: number }): number {
    // Mock APY calculation based on weights
    const usdcRate = 0.05; // 5% APY
    const usdtRate = 0.06; // 6% APY

    const usdcWeight = weights.usdc_weight_bps / 10000;
    const usdtWeight = weights.usdt_weight_bps / 10000;

    return (usdcWeight * usdcRate + usdtWeight * usdtRate) * 100;
  }

  static calculateVolatility(weights: { usdc_weight_bps: number; usdt_weight_bps: number }): number {
    // Mock volatility calculation
    const usdcVol = 0.02; // 2% volatility
    const usdtVol = 0.03; // 3% volatility

    const usdcWeight = weights.usdc_weight_bps / 10000;
    const usdtWeight = weights.usdt_weight_bps / 10000;

    return Math.sqrt(usdcWeight * usdcVol * usdcVol + usdtWeight * usdtVol * usdtVol) * 100;
  }
}

main().catch(console.error);
