use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Agent {
    pub agent_id: u64,
    pub owner: Pubkey,
    pub compute_node: Pubkey,
    pub public: bool,
    pub bump: u8,
}
