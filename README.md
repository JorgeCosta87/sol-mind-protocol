# Sol Mind Protocol

A Solana protocol built with Anchor that enables project management, NFT minting, and marketplace operations. The protocol consists of two interconnected programs: `sol-mind-protocol` for protocol and project management, and `nft-operations` for NFT creation and trading.

## Features

### Protocol Management (sol-mind-protocol)
- **Initialize Protocol**: Set up protocol configuration with admins, whitelisted addresses, and fee structure
- **Create Project**: Create a new project with configurable authorities and treasury
- **Update Fees**: Modify protocol fees for different operations (fixed or percentage-based)
- **Transfer Fees**: Transfer accumulated protocol and project fees

### NFT Operations (nft-operations)
- **Create Minter Config**: Configure NFT minting parameters including price, supply limits, and collection settings
- **Mint Asset**: Mint NFTs using MPL Core with configurable metadata and plugins
- **Create Trade Hub**: Set up a marketplace hub for NFT trading with custom fee rates
- **List Asset**: List NFTs for sale on a trade hub
- **Purchase Asset**: Buy listed NFTs with automatic fee distribution


## Instructions

### sol-mind-protocol

#### Initialize Protocol

Initializes the protocol with admin addresses, whitelisted transfer addresses, and fee structure.

**Parameters:**
- `admins`: List of admin public keys (max 3)
- `whitelist_transfer_addrs`: List of whitelisted addresses for PDA transfers (max 3)
- `fees`: Fee structure for all protocol operations

```rust
pub fn initialize_protocol(
    ctx: Context<InitializeProtocol>,
    admins: Vec<Pubkey>,
    whitelist_transfer_addrs: Vec<Pubkey>,
    fees: FeesStructure,
) -> Result<()>
```

**What it does:**
- Creates protocol config PDA account
- Sets admin addresses and whitelist
- Configures fee structure for all operations

#### Create Project

Creates a new project with a treasury account and configurable authorities.

**Parameters:**
- `project_id`: Unique identifier for the project
- `name`: Project name (max 64 characters)
- `description`: Project description (max 200 characters)
- `authorities`: List of authorized addresses (max 3)

```rust
pub fn create_project(
    ctx: Context<CreateProject>,
    project_id: u64,
    name: String,
    description: String,
    authorities: Vec<Pubkey>,
) -> Result<()>
```

**What it does:**
- Creates project config PDA account
- Creates treasury PDA account (rent-exempt)
- Pays protocol fee for project creation
- Transfers rent-exempt amount to treasury

#### Update Fees

Updates the entire fee structure or a single operation fee.

**Parameters:**
- `fees`: Complete fee structure (for full update)
- `operation`: Specific operation type (for single update)
- `fee`: Fee amount and type (Fixed or Percentage)

```rust
pub fn update_fees(ctx: Context<UpdateFees>, fees: FeesStructure) -> Result<()>

pub fn update_single_fee(
    ctx: Context<UpdateFees>,
    operation: Operation,
    fee: Fee,
) -> Result<()>
```

**What it does:**
- Validates admin authority
- Updates fee configuration in protocol config

#### Transfer Project Fees

Transfers accumulated fees from project treasury to specified address.

**Parameters:**
- `amount`: Amount to transfer (in lamports)

```rust
pub fn transfer_project_fees(ctx: Context<TransferProjectFees>, amount: u64) -> Result<()>
```

**What it does:**
- Validates project authority
- Transfers specified amount from treasury
- Ensures treasury remains rent-exempt

#### Transfer Protocol Fees

Transfers accumulated protocol fees to specified address.

**Parameters:**
- `amount`: Amount to transfer (in lamports)

```rust
pub fn transfer_protocol_fees(ctx: Context<ProtocolFeesTransfer>, amount: u64) -> Result<()>
```

**What it does:**
- Validates admin authority
- Transfers specified amount from protocol config
- Ensures protocol config remains rent-exempt

### nft-operations

#### Create Minter Config

Creates a configuration for NFT minting with price, supply limits, and optional collection.

**Parameters:**
- `name`: Minter configuration name (max 32 characters)
- `mint_price`: Price per NFT mint (in lamports)
- `max_supply`: Maximum number of NFTs (0 for unlimited)
- `assets_config`: Optional configuration for auto-generated asset names and URIs
- `uri`: Collection URI (required if collection is provided)
- `plugins`: Optional MPL Core plugins for collection

```rust
pub fn create_minter_config(
    ctx: Context<CreateMinterConfig>,
    name: String,
    mint_price: u64,
    max_supply: u64,
    assets_config: Option<AssetsConfig>,
    uri: Option<String>,
    plugins: Option<Vec<Vec<u8>>>,
) -> Result<()>
```

**What it does:**
- Creates minter config PDA account
- Pays protocol fee for minter config creation
- Optionally creates MPL Core collection if provided
- Sets collection authority to minter config PDA

#### Mint Asset

Mints a new NFT asset using MPL Core.

**Parameters:**
- `name`: Asset name (required if no assets_config in minter)
- `uri`: Asset URI (required if no assets_config in minter)
- `plugins`: Optional MPL Core plugins

```rust
pub fn mint_asset(
    ctx: Context<MintAsset>,
    name: Option<String>,
    uri: Option<String>,
    plugins: Option<Vec<Vec<u8>>>,
) -> Result<()>
```

**What it does:**
- Validates max supply limit
- Pays protocol fee for minting
- Creates MPL Core asset with specified metadata
- Increments mint counter
- Transfers mint price from payer to project treasury

#### Create Trade Hub

Creates a marketplace hub for NFT trading with configurable fees.

**Parameters:**
- `name`: Trade hub name (max 32 characters)
- `fee_bps`: Fee rate in basis points (1/10000)

```rust
pub fn create_trade_hub(
    ctx: Context<CreateTradeHub>,
    name: String,
    fee_bps: u64,
) -> Result<()>
```

**What it does:**
- Creates trade hub PDA account
- Pays protocol fee for trade hub creation
- Sets fee rate for marketplace transactions

#### List Asset

Lists an NFT for sale on a trade hub.

**Parameters:**
- `price`: Listing price (in lamports)

```rust
pub fn list_asset(ctx: Context<ListAsset>, price: u64) -> Result<()>
```

**What it does:**
- Creates listing PDA account
- Adds TransferDelegate plugin with trade hub as authority
- Adds FreezeDelegate plugin to prevent transfers while listed
- Validates asset ownership

#### Purchase Asset

Purchases a listed NFT and transfers ownership.

```rust
pub fn purchase_asset(ctx: Context<Purchase>) -> Result<()>
```

**What it does:**
- Pays protocol fee for trade
- Calculates and distributes fees (protocol fee + trade hub fee)
- Transfers remaining amount to seller
- Unfreezes asset
- Transfers asset ownership to buyer
- Uses trade hub PDA as transfer authority

## Account Structure

### Protocol Config

The protocol config PDA stores:
- `admins`: List of admin public keys (max 3)
- `whitelist_transfer_addrs`: Whitelisted addresses for PDA transfers (max 3)
- `fees`: Fee structure for all operations
- `bump`: PDA bump seed

**Seeds:** `["sol-mind-protocol"]`

### Project Config

The project config PDA stores:
- `protocol_config`: Protocol config public key
- `project_id`: Unique project identifier
- `owner`: Project owner public key
- `name`: Project name (max 64 characters)
- `description`: Project description (max 200 characters)
- `authorities`: List of authorized addresses (max 3)
- `treasury_bump`: Treasury PDA bump seed
- `bump`: Project config PDA bump seed

**Seeds:** `["project", owner, project_id.to_le_bytes(), protocol_config]`

### Treasury Account

A system account PDA owned by the project that holds project funds.

**Seeds:** `["treasury", project_config]`

### Minter Config

The minter config PDA stores:
- `name`: Configuration name (max 32 characters)
- `mint_price`: Price per NFT mint
- `mints_counter`: Current number of mints
- `max_supply`: Maximum supply (0 for unlimited)
- `assets_config`: Optional asset naming/URI configuration
- `collection`: Optional MPL Core collection public key
- `bump`: PDA bump seed

**Seeds:** `["minter_config", project_config, name]`

### Trade Hub

The trade hub PDA stores:
- `project`: Project config public key
- `name`: Trade hub name (max 32 characters)
- `fee_bps`: Fee rate in basis points
- `bump`: PDA bump seed

**Seeds:** `["trade_hub", name, project_config]`

### Listing

The listing PDA stores:
- `owner`: Asset owner public key
- `asset`: MPL Core asset public key
- `price`: Listing price (in lamports)
- `created_at`: Unix timestamp of listing creation
- `bump`: PDA bump seed

**Seeds:** `["listing", asset, trade_hub]`

## Fee Structure

The protocol supports two fee types:
- **Fixed**: Flat fee amount in lamports
- **Percentage**: Fee calculated as percentage of transaction amount (stored as basis points)

Supported operations:
- `CreateProject`: Fee for creating a project
- `CreateMinterConfig`: Fee for creating a minter configuration
- `CreateTradeHub`: Fee for creating a trade hub
- `TradeNFT`: Fee for NFT purchases (can be percentage-based)
- `MintAsset`: Fee for minting NFTs
- `GenericOperation`: Default fee for other operations

## Setup

### Prerequisites

- Anchor installed (v0.32.1)
- Solana CLI installed
- Node.js and yarn/npm
- Rust toolchain

### Installation

```bash
# Install dependencies
yarn install

# Build the programs
anchor build

# Run tests
anchor test
```

### Build and Generate Clients

```bash
# Build programs and generate clients
yarn build

# Generate clients only
yarn generate
```

### Testing

Tests are written in Rust using **LiteSVM** for fast, in-process Solana program execution without requiring a local validator.

Run all tests:
```bash
anchor test
```

Run tests for specific program:
```bash
anchor test --program-name sol_mind_protocol
anchor test --program-name nft_operations
```

#### Test Structure

Tests are organized into separate files for each program:
- `tests/sol-mind-protocol.rs`: Tests for protocol management operations
- `tests/nft-operations.rs`: Tests for NFT operations

The test suite uses a modular structure:

- **TestFixture**: Main test fixture struct that initializes LiteSVM, deploys programs, and manages test accounts. Provides builder methods like `with_initialize_protocol()` and `with_initialize_project()` for test setup.

- **Instructions**: Helper module that wraps instruction building and transaction execution using Anchor client builders.

- **AccountHelper**: Utility module for finding PDAs and deserializing account data from the SVM state.

- **test_data**: Constants and default test data used across tests.

Each test follows a pattern:
1. Create a `TestFixture` with required setup (protocol/project initialization)
2. Execute instructions using the `Instructions` helper
3. Verify account state using `AccountHelper` methods
4. Assert expected outcomes

## Architecture

The protocol consists of two interconnected programs:

1. **sol-mind-protocol**: Core protocol management
   - Handles protocol-wide configuration
   - Manages projects and their treasuries
   - Enforces fee collection and distribution

2. **nft-operations**: NFT-specific operations
   - Integrates with MPL Core for NFT creation
   - Manages minting configurations
   - Provides marketplace functionality

