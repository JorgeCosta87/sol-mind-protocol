use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Pending,
    Processing,
}

#[account]
#[derive(InitSpace)]
pub struct TaskData {
    pub compute_node: Pubkey,
    #[max_len(200)]
    pub data: Vec<u8>,
    pub status: TaskStatus,
    #[max_len(200)]
    pub result: Vec<u8>,
    pub bump: u8,
}
