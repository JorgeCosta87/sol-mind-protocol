// Program paths and IDs
pub const SOL_MIND_PROTOCOL_KEYPAIR_PATH: &str = "target/deploy/sol_mind_protocol-keypair.json";
pub const SOL_MIND_PROTOCOL_SO_PATH: &str = "target/deploy/sol_mind_protocol.so";
pub const NFT_OPERATIONS_KEYPAIR_PATH: &str = "target/deploy/nft_operations-keypair.json";
pub const NFT_OPERATIONS_SO_PATH: &str = "target/deploy/nft_operations.so";

pub const MPL_CORE_PROGRAM_ID: &str = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";
pub const MPL_CORE_PROGRAM_SO_PATH: &str = "tests/programs/mpl_core.so";

// Test data constants
pub const PROJECT_1_ID: u64 = 1u64;
pub const DEFAULT_PROJECT_NAME: &str = "Test project";
pub const DEFAULT_PROJECT_DESCRIPTION: &str = "Project description";

pub const MINTER_NAME: &str = "Minter";
pub const MINT_PRICE: u64 = 1_000_000_000;
pub const MAX_SUPPLY: u64 = 5;
pub const COLLECTION_URI: &str = "https://";

pub const ASSET_NAME: &str = "Test Asset";
pub const ASSET_URI: &str = "https://";

pub const TRADE_HUB_NAME: &str = "Test Trade Hub";
pub const TRADE_HUB_FEE_BPS: u64 = 350;
pub const LISTING_PRICE: u64 = 100_000_000;

pub const FEE_CREATE_PROJECT_AMOUNT: u64 = 1_000_000;
pub const FEE_CREATE_MINTER_CONFIG_AMOUNT: u64 = 500_000;
pub const FEE_MINT_ASSET_AMOUNT: u64 = 50_000;
pub const FEE_GENERIC_OPERATION_AMOUNT: u64 = 100_000;
pub const FEE_CREATE_TRADE_HUB_AMOUNT: u64 = 200_000;
pub const FEE_TRADE_NFT_AMOUNT: u64 = 25_000;

pub fn default_fees_structure() -> sol_mind_protocol_client::types::FeesStructure {
    use sol_mind_protocol_client::types::{Fee, FeeType};
    sol_mind_protocol_client::types::FeesStructure {
        create_project: Fee {
            amount: FEE_CREATE_PROJECT_AMOUNT,
            fee_type: FeeType::Fixed,
        },
        create_minter_config: Fee {
            amount: FEE_CREATE_MINTER_CONFIG_AMOUNT,
            fee_type: FeeType::Fixed,
        },
        mint_asset: Fee {
            amount: FEE_MINT_ASSET_AMOUNT,
            fee_type: FeeType::Fixed,
        },
        generic_operation: Fee {
            amount: FEE_GENERIC_OPERATION_AMOUNT,
            fee_type: FeeType::Fixed,
        },
        create_trade_hub: Fee {
            amount: FEE_CREATE_TRADE_HUB_AMOUNT,
            fee_type: FeeType::Fixed,
        },
        trade_nft: Fee {
            amount: FEE_TRADE_NFT_AMOUNT,
            fee_type: FeeType::Fixed,
        },
    }
}
