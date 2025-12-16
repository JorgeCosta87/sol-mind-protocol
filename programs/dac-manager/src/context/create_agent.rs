use anchor_lang::prelude::*;

use crate::ActionType;
use crate::utils::init_dynamic_pda;
use crate::{
    AgentStatus, Goal, GoalStatus, TaskData, errors::ErrorCode, state::{Agent, ComputeNodeInfo, ComputeNodeStatus, TaskStatus}
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
        allocated_goals: u32,
        allocated_tasks: u32,
        remaining_accounts: &[AccountInfo<'info>],
        bumps: &CreateAgentBumps,
    ) -> Result<()> {
        self.agent.set_inner(Agent {
            agent_id,
            owner: self.owner.key(),
            compute_node: self.compute_node_info.node_pubkey,
            public,
            status: AgentStatus::Pending,
            allocated_goals,
            allocated_tasks,
            bump: bumps.agent,
        });

        let mut remaining_accounts_iter = remaining_accounts.iter();
        let agent_key = self.agent.key();
        for goal_id in 0..allocated_goals {
            let goal_account_info = remaining_accounts_iter.next()
                .ok_or(ErrorCode::MissingAccount)?;

            let seeds = &[
                b"goal",
                agent_key.as_ref(),
                &goal_id.to_le_bytes(),
            ];

            let bump = init_dynamic_pda(
                &self.payer,
                goal_account_info,
                seeds,
                8 + Goal::INIT_SPACE,
                &crate::ID,
                &self.system_program,
            )?;

            let goal_data = Goal {
                goal_index: goal_id,
                owner: self.owner.key(),
                agent: self.agent.key(),
                description: "".to_string(),
                status: GoalStatus::Pending,
                max_iterations: 0,
                current_iteration: 0,
                bump: bump,
            };
            
            goal_data.try_serialize(&mut *goal_account_info.try_borrow_mut_data()?)?;
        }

        for task_id in 0..allocated_tasks {
            let task_account_info = remaining_accounts_iter.next()
                .ok_or(ErrorCode::MissingAccount)?;

            let seeds = &[
                b"task_data",
                agent_key.as_ref(),
                &task_id.to_le_bytes(),
            ];

            let bump = init_dynamic_pda(
                &self.payer,
                task_account_info,
                seeds,
                8 + TaskData::INIT_SPACE,
                &crate::ID,
                &self.system_program,
            )?;

            let task_data = TaskData {
                task_index: task_id,
                goal: Pubkey::default(),
                action_type: ActionType::Llm,
                status: TaskStatus::Ready,
                data: vec![],
                result: vec![],
                bump,
            };

            task_data.try_serialize(&mut *task_account_info.try_borrow_mut_data()?)?;
        }

        Ok(())
    }
}
