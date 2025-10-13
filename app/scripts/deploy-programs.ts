import { execSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';

// Minimal helper to run Anchor deploy commands from the repository root.
function runCommand(command: string) {
  execSync(command, {
    stdio: 'inherit',
    cwd: path.resolve(__dirname, '..', '..'),
    env: process.env,
  });
}

function main() {
  try {
    runCommand('anchor build');
    runCommand('anchor deploy --provider.cluster devnet');
  } catch (error) {
    if (error instanceof Error) {
      console.error('Failed to deploy Anchor programs:', error.message);
    }
    process.exitCode = 1;
  }
}

main();
