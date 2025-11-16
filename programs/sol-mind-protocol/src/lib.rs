use anchor_lang::prelude::*;

declare_id!("EkK8DLYGgXKi1Hcp5xpoyrkgMqxE6MqyhQh35QFACJ24");

pub mod context;
pub mod errors;
pub mod helpers;
pub mod state;

pub use context::*;
pub use helpers::*;
pub use state::*;

#[program]
pub mod sol_mind_protocol {
    use super::*;
    pub fn initialize_project(
        ctx: Context<InitializeProject>,
        project_id: u64,
        name: String,
        description: String,
        treasury: Pubkey,
        authorities: Vec<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.initialize_project(
            project_id,
            name,
            description,
            treasury,
            authorities,
            ctx.bumps.project_config,
        )
    }

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
}
