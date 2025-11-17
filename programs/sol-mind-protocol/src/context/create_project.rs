use anchor_lang::prelude::*;

use crate::{
    helpers::pay_protocol_fee, 
    state::{
        Operation, ProjectConfig, ProtocolConfig
    }
};

#[derive(Accounts)]
#[instruction(project_id: u64)]
pub struct CreateProject<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + ProjectConfig::INIT_SPACE,
        seeds = [
            b"project",
            owner.key.as_ref(),
            project_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub project_config: Account<'info, ProjectConfig>,
    #[account(
        mut,
        seeds = [b"sol-mind-protocol"],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateProject<'info> {
    pub fn create_project(
        &mut self,
        project_id: u64,
        name: String,
        description: String,
        authorities: Vec<Pubkey>,
        bump: u8,
    ) -> Result<()> {
        pay_protocol_fee(
            &self.owner,
            &self.protocol_config,
            Operation::CreateProject,
            None,
            &self.system_program,
        )?;

        self.project_config.set_inner(ProjectConfig {
            project_id,
            owner: self.owner.key(),
            name,
            description,
            minter_config_counter: 0,
            autthorities: authorities,
            bump,
        });

        Ok(())
    }
}
