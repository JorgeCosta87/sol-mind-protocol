use anchor_lang::prelude::*;

use crate::{
    errors::ErrorCode, state::{Agent, AgentStatus}
};

#[derive(Accounts)]
#[instruction(agent_id: u64)]
pub struct ActivateAgent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub compute_node: Signer<'info>,
    #[account(
        mut,
        has_one = compute_node.key() @ ErrorCode::Unauthorized,
        seeds = [b"agent", agent.owner.as_ref(), agent_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,
}

impl<'info> ActivateAgent<'info> {
    pub fn activate_agent(
        &mut self,
    ) -> Result<()> {
        self.agent.status = AgentStatus::Active;
        Ok(())
    }
}
