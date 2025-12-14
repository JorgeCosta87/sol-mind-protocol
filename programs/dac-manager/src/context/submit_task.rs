use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{Agent, TaskData, TaskStatus};

#[derive(Accounts)]
pub struct SubmitTask<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub submitter: Signer<'info>,
    #[account(
        mut,
        seeds = [b"task_data", agent.key().as_ref()],
        bump = task_data.bump,
    )]
    pub task_data: Account<'info, TaskData>,
    #[account(
        mut,
        seeds = [b"agent", agent.owner.as_ref(), agent.agent_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    pub system_program: Program<'info, System>,
}

impl<'info> SubmitTask<'info> {
    pub fn submit_task(&mut self, data: Vec<u8>) -> Result<()> {
        require!(
            self.submitter.key() == self.agent.owner || self.submitter.key() == self.agent.compute_node,
            ErrorCode::Unauthorized
        );
        require!(
            self.task_data.status == TaskStatus::Ready,
            ErrorCode::InvalidTaskStatus
        );

        self.task_data.set_inner(TaskData {
            compute_node: self.agent.compute_node,
            data: data,
            status: TaskStatus::Pending,
            result: vec![],
            bump: self.task_data.bump,
        });

        Ok(())
    }
}
