use anchor_lang::prelude::*;

declare_id!("7kB6TUSc5NtpMcStvWmcyPuiUhTXPkHxKHLbzcqu74MC");

pub mod context;
pub mod state;
pub mod errors;
pub mod helpers;

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
}

