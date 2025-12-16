use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum GoalStatus {
    Pending,
    Active,
    Completed,
}

#[account]
#[derive(InitSpace)]
pub struct Goal {
    pub goal_index: u32,
    pub owner: Pubkey, //to support mutiple goals in the future
    pub agent: Pubkey,
    pub status: GoalStatus,
    #[max_len(200)]
    pub description: String,
    pub max_iterations: u64,
    pub current_iteration: u64,
    pub bump: u8,
}