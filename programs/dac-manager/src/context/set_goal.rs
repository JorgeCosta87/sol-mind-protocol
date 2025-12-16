use anchor_lang::prelude::*;

use crate::{Agent, Goal, GoalStatus, errors::ErrorCode};

#[derive(Accounts)]
#[instruction(goal_index: u32)]
pub struct SetGoal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = agent.key() @ ErrorCode::Unauthorized,
        seeds = [b"goal", agent.key().as_ref(), goal_index.to_le_bytes().as_ref()],
        bump = goal.bump,
    )]
    pub goal: Account<'info, Goal>,
    #[account(
        mut,
        seeds = [b"agent", agent.owner.as_ref(), agent.agent_id.to_le_bytes().as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,
    pub system_program: Program<'info, System>,
}

impl<'info> SetGoal<'info> {
    pub fn set_goal(&mut self, description: String, max_iterations: u64) -> Result<()> {
        self.goal.owner = self.owner.key();
        self.goal.agent = self.agent.key();
        self.goal.status = GoalStatus::Active;
        self.goal.description = description;
        self.goal.max_iterations = max_iterations;
        self.goal.current_iteration = 0;

        Ok(())
    }
}