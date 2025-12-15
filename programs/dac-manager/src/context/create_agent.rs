use anchor_lang::prelude::*;

use crate::{
    AgentStatus, TaskData, errors::ErrorCode, state::{Agent, ComputeNodeInfo, ComputeNodeStatus, TaskStatus}
};

#[derive(Accounts)]
#[instruction(agent_id: u64)]
pub struct CreateAgent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Agent::INIT_SPACE,
        seeds = [b"agent", owner.key().as_ref(), agent_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub agent: Account<'info, Agent>,
    #[account(
        init,
        payer = payer,
        space = 8 + TaskData::INIT_SPACE,
        seeds = [b"task_data", agent.key().as_ref()],
        bump,
    )]
    pub task_data: Account<'info, TaskData>,
    #[account(
        constraint = compute_node_info.status == ComputeNodeStatus::Approved @ ErrorCode::ComputeNodeNotApproved,
        seeds = [
            b"compute_node",
            compute_node_info.node_pubkey.as_ref()
        ],
        bump = compute_node_info.bump,
    )]
    pub compute_node_info: Account<'info, ComputeNodeInfo>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateAgent<'info> {
    pub fn create_agent(
        &mut self,
        agent_id: u64,
        public: bool,
        bumps: &CreateAgentBumps,
    ) -> Result<()> {
        self.agent.set_inner(Agent {
            agent_id,
            owner: self.owner.key(),
            compute_node: self.compute_node_info.node_pubkey,
            public,
            status: AgentStatus::Pending,
            bump: bumps.agent,
        });

        self.task_data.set_inner(TaskData {
            compute_node: self.compute_node_info.node_pubkey,
            data: vec![],
            status: TaskStatus::Ready,
            result: vec![],
            bump: bumps.task_data,
        });

        Ok(())
    }
}
