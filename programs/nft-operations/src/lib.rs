use anchor_lang::prelude::*;

declare_id!("Anv3tR3s1eQqQE1a5wnhtYdi6dkjv9rTn34V6pom9C9q");

pub mod context;
pub mod errors;
pub mod helpers;
pub mod state;

pub use context::*;
pub use helpers::*;
pub use state::*;

#[program]
pub mod nft_operations {
    use super::*;
    pub fn create_minter_config(
        ctx: Context<CreateMinterConfig>,
        name: String,
        mint_price: u64,
        max_supply: u64,
        assets_config: Option<AssetsConfig>,
        uri: Option<String>,
        plugins: Option<Vec<Vec<u8>>>,
    ) -> Result<()> {
        let decoded_plugins = decoded_core_plugins(plugins)?;
        ctx.accounts.create_minter_config(
            name,
            mint_price,
            max_supply,
            assets_config,
            uri,
            decoded_plugins,
            &ctx.bumps,
        )
    }

    pub fn mint_asset(
        ctx: Context<MintAsset>,
        name: Option<String>,
        uri: Option<String>,
        plugins: Option<Vec<Vec<u8>>>,
    ) -> Result<()> {
        let decoded_plugins = decoded_core_plugins(plugins)?;

        ctx.accounts.mint_asset(name, uri, decoded_plugins)
    }

    pub fn create_trade_hub(
        ctx: Context<CreateTradeHub>,
        name: String,
        fee_bps: u64,
    ) -> Result<()> {
        ctx.accounts
            .create_trade_hub(name, fee_bps, ctx.bumps.trade_hub)
    }

    pub fn list_asset(ctx: Context<ListAsset>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, ctx.bumps.listing)
    }

    pub fn delist_asset(ctx: Context<DelistAsset>) -> Result<()> {
        ctx.accounts.delist_asset()
    }

    pub fn purchase_asset(ctx: Context<Purchase>, max_price: u64) -> Result<()> {
        ctx.accounts.purchase_asset(max_price)
    }
}
