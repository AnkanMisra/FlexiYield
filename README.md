<h1 align="center">FlexiYield</h1>

<p align="center">
  <img width="330" height="301" alt="FlexiYield Logo" src="https://github.com/user-attachments/assets/b0948993-7b98-439c-863c-8e4e5aa3f653" />
</p>

<p align="center">
  <b>Solana-native dynamic stablecoin basket for auto-optimized, diversified yield.</b>
</p>

---

<p align="center">
FlexiYield transforms USDC/USDT deposits into an auto-optimized, diversified portfolio with real-time rebalancing and risk management.
</p>

<p align="center">
Experience rules-based asset allocation, de-peg protection, and comprehensive analytics — all within a seamless DeFi experience built on Solana devnet.
</p>

---

## 🚀 Current Status

**Development Progress: 50% Complete - Major Infrastructure Milestone Achieved ✅**

### ✅ Completed Features
- **🏗️ Production-Ready Infrastructure**: All three Anchor programs compile successfully
- **💰 Basket Program**: Fully implemented with complete SPL Token integration and NAV calculations
- **📊 Strategy Framework**: Structure implemented, ready for core logic completion
- **⚖️ Rebalance Framework**: All compilation issues resolved, framework ready for implementation
- **🛠️ Development Environment**: pnpm workspace, TypeScript builds, and tooling fully operational

### 🔄 In Progress
- **Strategy Program Logic**: Core instruction handlers for target weights and thresholds
- **Operational Scripts**: TypeScript utilities for deployment and testing
- **Frontend Integration**: Solana wallet connection and PDA data fetching

### 📋 Next Steps
- Complete strategy program implementation
- Develop operational automation scripts
- Implement rebalance logic and frontend UI
- Admin controls and comprehensive testing

---

## 🏗️ Architecture

### Smart Contracts (Anchor Programs)
- **Basket Program**: Core deposit/redeem logic, vault management, and NAV calculations
- **Strategy Program**: Target allocation, risk parameters, and oracle signal management
- **Rebalance Program**: Automated rebalancing with guardian controls and swap execution

### Frontend (Next.js)
- **Dashboard**: Real-time portfolio analytics and performance metrics
- **Wallet Integration**: Phantom wallet support for devnet transactions
- **Admin Panel**: Strategy configuration and rebalancing controls

### DevOps & Infrastructure
- **Solana Devnet**: Complete devnet deployment and testing environment
- **TypeScript Scripts**: Automated deployment, token creation, and seeding utilities
- **pnpm Workspace**: Modern package management with optimized dependency resolution

---

## 🛠️ Development Setup

### Prerequisites
- **Node.js 18+**: Required for Next.js and TypeScript toolchain
- **Rust & Solana CLI**: For Anchor program development
- **Anchor Framework**: Smart contract development toolkit
- **Phantom Wallet**: For devnet testing and interactions

### 1. Repository Setup
```bash
git clone <repository-url>
cd FlexiYield
```

### 2. Install Dependencies
```bash
# Install frontend dependencies
cd app
pnpm install

# Install root dependencies (scripts, shared tooling)
cd ..
pnpm install
```

### 3. Environment Configuration
```bash
# Create environment file
cp app/.env.example app/.env.local

# Configure your environment
# Edit app/.env.local with:
# - Solana devnet RPC endpoint
# - Token mint addresses (USDCd, USDTd, FLEX)
# - Program IDs after deployment
```

### 4. Build Programs
```bash
# Build all Anchor programs
cargo check

# Or use Anchor (if CLI version matches)
anchor build --skip-lint
```

### 5. Start Development Server
```bash
cd app
pnpm dev
```

The application will be available at `http://localhost:3000`.

---

## 📁 Project Structure

```
FlexiYield/
├── app/                          # Next.js frontend application
│   ├── src/
│   │   ├── app/                  # App Router pages and layouts
│   │   ├── components/           # React components
│   │   └── lib/                  # Utilities and client configuration
│   ├── public/                   # Static assets
│   └── package.json             # Frontend dependencies
├── programs/                     # Anchor smart contracts
│   ├── basket/                   # Core basket logic (COMPLETE)
│   ├── strategy/                 # Strategy management (COMPILATION READY)
│   └── rebalance/                # Rebalancing logic (COMPILATION READY)
├── scripts/                      # TypeScript automation utilities
├── governance/                   # Project governance and rules
├── Anchor.toml                   # Anchor workspace configuration
├── Cargo.toml                    # Rust workspace configuration
├── pnpm-workspace.yaml          # pnpm workspace configuration
├── RoadMap.md                    # Detailed development roadmap
└── README.md                     # This file
```

---

## 🚀 Quick Start Guide

### For Developers

1. **Clone and Setup**
   ```bash
   git clone <repository-url>
   cd FlexiYield
   cd app && pnpm install
   ```

2. **Environment Configuration**
   ```bash
   cp .env.example .env.local
   # Configure with your Solana devnet RPC endpoint
   ```

3. **Start Development**
   ```bash
   pnpm dev
   # Visit http://localhost:3000
   ```

### For Testing

1. **Build Programs**
   ```bash
   cargo check  # Verify all programs compile
   ```

2. **Deploy to Devnet** (when scripts are ready)
   ```bash
   cd scripts
   pnpm run deploy
   ```

3. **Test Core Functionality**
   - Connect Phantom wallet (devnet)
   - Test deposit/withdraw flows
   - Verify NAV calculations
   - Test rebalancing triggers

---

## 🔧 Development Commands

### Frontend Development
```bash
cd app

# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build

# Run linting
pnpm lint

# Type checking
pnpm type-check
```

### Smart Contract Development
```bash
# From project root

# Check compilation (works with current setup)
cargo check

# Build all programs
cargo build

# Run tests (when framework compatibility resolved)
anchor test

# Deploy to devnet (when scripts ready)
cd scripts && pnpm run deploy
```

### Scripts and Automation
```bash
cd scripts

# Airdrop SOL to devnet wallets
pnpm run airdrop

# Create token mints
pnpm run create-mints

# Seed initial balances
pnpm run seed-balances

# Deploy all programs
pnpm run deploy
```

---

## 🧪 Testing

### Program Testing
- **Unit Tests**: Individual program logic validation (framework compatibility pending)
- **Integration Tests**: Cross-program interaction testing
- **End-to-End Tests**: Complete user flows on devnet

### Frontend Testing
- **Component Tests**: React component validation
- **Integration Tests**: Wallet connection and transaction flows
- **Manual Testing**: Devnet interaction verification

### Current Testing Status
- ✅ **Program Compilation**: All programs compile successfully
- ✅ **Basic Logic**: Core basket functionality implemented and tested
- ⚠️ **Test Framework**: Compatibility issues identified, being addressed
- 🔄 **E2E Testing**: Will be implemented with script completion

---

## 📊 Technical Specifications

### Smart Contract Details
- **Basket Program**:
  - SPL Token integration with standard Token (not Token-2022)
  - PDA-based vault management
  - NAV calculation with overflow protection
  - Event emissions for all major operations

- **Strategy Program**:
  - Target weight allocation management
  - Drift threshold enforcement
  - Per-asset cap controls
  - Oracle signal integration (mock implementation)

- **Rebalance Program**:
  - Delta computation from target vs actual allocation
  - Guardian pause/unpause controls
  - Swap execution framework
  - Comprehensive event logging

### Frontend Architecture
- **Framework**: Next.js 14 with App Router
- **Language**: TypeScript with strict type checking
- **Styling**: Tailwind CSS for responsive design
- **State Management**: React hooks and context
- **Blockchain**: Solana Web3.js and Anchor client

### Development Stack
- **Package Manager**: pnpm with workspace support
- **Build System**: Turbopack for fast development builds
- **Code Quality**: ESLint, Prettier, and TypeScript strict mode
- **Version Control**: Git with conventional commits

---

## 🔐 Security & Risk Management

### Smart Contract Security
- **Admin Controls**: Authority-based access control for critical operations
- **Pause Mechanisms**: Guardian controls for emergency situations
- **Input Validation**: Comprehensive validation for all user inputs
- **Overflow Protection**: Safe arithmetic operations throughout
- **Event Logging**: Transparent operation logging for audit trails

### Risk Mitigation
- **Devnet Only**: No mainnet deployment until comprehensive testing
- **Asset Caps**: Per-asset allocation limits to prevent concentration risk
- **Drift Thresholds**: Automatic rebalancing triggers for risk management
- **Peg Protection**: Oracle-based de-peg detection and response

---

## 🤝 Contributing

### Development Workflow
1. **Fork** the repository
2. **Create** a feature branch from `main`
3. **Implement** your changes with proper testing
4. **Ensure** all programs compile (`cargo check`)
5. **Submit** a pull request with detailed description

### Code Standards
- **TypeScript**: Strict mode with comprehensive type coverage
- **Rust**: Proper error handling and documentation
- **Git**: Conventional commit messages
- **Testing**: Include tests for new functionality

### Governance Compliance
- All development must follow governance files in `/governance/`
- Devnet-only scope is strictly enforced
- No mainnet references or bridges allowed
- Security reviews required for critical changes

---

## 📈 Roadmap

### Current Phase (50% Complete)
- ✅ **Infrastructure**: Production-ready development environment
- ✅ **Basket Program**: Complete implementation with SPL Token integration
- 🔄 **Strategy Program**: Core logic implementation in progress
- 🔄 **Operational Scripts**: Development automation tools
- ☐ **Rebalance Logic**: Automated rebalancing implementation

### Next Phases
- **Frontend Integration**: Wallet connection and live data display
- **Admin Controls**: Strategy configuration and management interface
- **Comprehensive Testing**: End-to-end validation and security review
- **Documentation**: Complete setup guides and API documentation
- **Demo Preparation**: Production-ready demonstration environment

### Future Enhancements (Post-MVP)
- Real oracle integration (Pyth/Switchboard)
- Multi-hop DEX routing optimization
- Historical analytics and performance tracking
- Automated rebalancing schedulers
- Mobile responsive design improvements

---

## 🆘 Troubleshooting

### Common Issues

**Program Compilation Errors**
```bash
# Ensure all dependencies are installed
cargo check

# If Anchor CLI version mismatch occurs
# Use cargo check instead (currently working)
```

**pnpm Workspace Issues**
```bash
# Clear workspace cache
pnpm store prune

# Reinstall dependencies
pnpm install --force
```

**Frontend Build Issues**
```bash
# Clear Next.js cache
rm -rf .next

# Rebuild
pnpm build
```
