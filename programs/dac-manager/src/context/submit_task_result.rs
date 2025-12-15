use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{Agent, TaskData, TaskStatus};

#[derive(Accounts)]
pub struct SubmitTaskResult<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub compute_node: Signer<'info>,
    #[account(
        mut,
        seeds = [b"task_data", agent.key().as_ref()],
        bump = task_data.bump,
    )]
    pub task_data: Account<'info, TaskData>,
    #[account(
        mut,
        has_one = compute_node.key() @ ErrorCode::Unauthorized,
        seeds = [b"agent", agent.owner.as_ref(), agent.agent_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    pub system_program: Program<'info, System>,
}

impl<'info> SubmitTaskResult<'info> {
    pub fn claim_task(&mut self) -> Result<()> {
        require!(
            self.task_data.status == TaskStatus::Pending,
            ErrorCode::InvalidTaskStatus
        );

        self.task_data.status = TaskStatus::Processing;
        self.task_data.compute_node = self.compute_node.key();

        Ok(())
    }

    pub fn submit_task_result(&mut self, result: Vec<u8>) -> Result<()> {
        require!(
            self.task_data.status == TaskStatus::Processing,
            ErrorCode::InvalidTaskStatus
        );

        self.task_data.result = result;
        self.task_data.status = TaskStatus::Ready;

        Ok(())
    }
}
