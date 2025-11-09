use anchor_lang::prelude::*;

declare_id!("8xdKnVNCbDX2GrRtbcGpsoZnyK4hEyCEdP6AojoDsVyw");

pub mod context;
pub mod state;

pub use context::*;
pub use state::*;

#[program]
pub mod sol_mind_protocol {
    use super::*;

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
        mint_price: u64,
        royalties: u16,
        max_supply: u64,
        //assets_config: Option<AssetsConfig>,
    ) -> Result<()> {
        ctx.accounts.create_collection(
            name, uri, mint_price, royalties, max_supply, &ctx.bumps
        )
    }
}
