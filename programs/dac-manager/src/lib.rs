use anchor_lang::prelude::*;

declare_id!("DeXj8mQDYUnLC2mX5xiRqgvYt193sBhJWRZTBkRLg79M");

pub mod context;
pub mod errors;
pub mod state;

pub use context::*;
pub use state::*;

#[program]
pub mod dac_manager {
    use super::*;

    pub fn register_compute_node(
        ctx: Context<RegisterComputeNode>,
        node_pubkey: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .register_compute_node(node_pubkey, &ctx.bumps)
    }

    pub fn claim_compute_node(ctx: Context<ClaimComputeNode>, node_info_cid: String) -> Result<()> {
        ctx.accounts.claim_compute_node(node_info_cid)
    }

    pub fn create_agent(
        ctx: Context<CreateAgent>,
        agent_id: u64,
        public: bool,
    ) -> Result<()> {
        ctx.accounts
            .create_agent(agent_id, public, &ctx.bumps)
    }

    pub fn submit_task(ctx: Context<SubmitTask>, data: Vec<u8>) -> Result<()> {
        ctx.accounts.submit_task(data)
    }

    pub fn claim_task(ctx: Context<SubmitTaskResult>) -> Result<()> {
        ctx.accounts.claim_task()
    }

    pub fn submit_task_result(ctx: Context<SubmitTaskResult>, result: Vec<u8>) -> Result<()> {
        ctx.accounts.submit_task_result(result)
    }
}
