use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum AgentStatus {
    Pending,
    Active,
    Inactive,
}

#[account]
#[derive(InitSpace)]
pub struct Agent {
    pub agent_id: u64,
    pub owner: Pubkey,
    pub compute_node: Pubkey,
    pub public: bool,
    pub status: AgentStatus,
    pub allocated_goals: u32,
    pub allocated_tasks: u32,
    pub bump: u8,
}