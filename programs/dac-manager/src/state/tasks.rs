use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Pending,
    Processing,
    AwaitingValidation,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ActionType {
    Llm,
    Tool(Pubkey),
    Agent2Agent(Pubkey),
}

#[account]
#[derive(InitSpace)]
pub struct TaskData {
    pub task_index: u32,
    pub goal: Pubkey,
    pub action_type: ActionType,
    pub status: TaskStatus,
    #[max_len(200)]
    pub data: Vec<u8>,
    #[max_len(200)]
    pub result: Vec<u8>,
    pub bump: u8,
}
