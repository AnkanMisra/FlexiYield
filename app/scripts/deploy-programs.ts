import { execSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import process from 'node:process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

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
