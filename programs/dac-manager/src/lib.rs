use anchor_lang::prelude::*;

declare_id!("DeXj8mQDYUnLC2mX5xiRqgvYt193sBhJWRZTBkRLg79M");

pub mod context;
pub mod errors;
pub mod state;
pub mod utils;

pub use context::*;
pub use state::*;

#[program]
pub mod dac_manager {
    use super::*;

    pub fn register_compute_node(
        ctx: Context<RegisterComputeNode>,
        node_pubkey: Pubkey,
    ) -> Result<()> {
        ctx.accounts.register_compute_node(node_pubkey, &ctx.bumps)
    }

    pub fn claim_compute_node(ctx: Context<ClaimComputeNode>, node_info_cid: String) -> Result<()> {
        ctx.accounts.claim_compute_node(node_info_cid)
    }

    pub fn create_agent<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateAgent<'info>>, 
        agent_id: u64, 
        public: bool, 
        allocated_goals: u32,
        allocated_tasks: u32,
    ) -> Result<()> {
        ctx.accounts.create_agent(
            agent_id, public, allocated_goals, allocated_tasks, &ctx.remaining_accounts, &ctx.bumps,
        )
    }
    
    pub fn activate_agent(ctx: Context<ActivateAgent>, agent_id: u64) -> Result<()> {
        ctx.accounts.activate_agent()
    }

    pub fn submit_task(ctx: Context<SubmitTask>, task_index: u32, data: Vec<u8>) -> Result<()> {
        ctx.accounts.submit_task(data)
    }

    pub fn claim_task(ctx: Context<SubmitTaskResult>) -> Result<()> {
        ctx.accounts.claim_task()
    }

    pub fn submit_task_result(ctx: Context<SubmitTaskResult>, result: Vec<u8>) -> Result<()> {
        ctx.accounts.submit_task_result(result)
    }

    pub fn set_goal(ctx: Context<SetGoal>, goal_index: u32, description: String, max_iterations: u64) -> Result<()> {
        ctx.accounts.set_goal(description, max_iterations)
    }
}
