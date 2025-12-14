use anchor_lang::prelude::*;

declare_id!("7GoSe63kVLi3Bt2mkPHUFVbmTcsZvp7QEUHjDB6GvLgV");

pub mod context;
pub mod errors;
pub mod helpers;
pub mod state;

pub use context::*;
pub use errors::*;
pub use helpers::*;
pub use state::*;

#[program]
pub mod sol_mind_protocol {
    use super::*;
    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        admins: Vec<Pubkey>,
        whitelist_transfer_addrs: Vec<Pubkey>,
        fees: FeesStructure,
    ) -> Result<()> {
        ctx.accounts.initialize_protocol(
            admins,
            whitelist_transfer_addrs,
            fees,
            ctx.bumps.protocol_config,
        )
    }

    pub fn update_fees(ctx: Context<UpdateFees>, fees: FeesStructure) -> Result<()> {
        ctx.accounts.update_fees(fees)
    }

    pub fn update_single_fee(
        ctx: Context<UpdateFees>,
        operation: Operation,
        fee: Fee,
    ) -> Result<()> {
        ctx.accounts.update_single_fee(operation, fee)
    }

    pub fn create_project(
        ctx: Context<CreateProject>,
        project_id: u64,
        name: String,
        description: String,
        authorities: Vec<Pubkey>,
    ) -> Result<()> {
        ctx.accounts
            .create_project(project_id, name, description, authorities, &ctx.bumps)
    }

    pub fn transfer_project_fees(ctx: Context<TransferProjectFees>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_project_fees(amount)
    }

    pub fn transfer_protocol_fees(ctx: Context<ProtocolFeesTransfer>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_protocol_fees(amount, &ctx.bumps)
    }
}
