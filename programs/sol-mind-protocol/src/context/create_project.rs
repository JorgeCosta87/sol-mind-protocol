use crate::{
    helpers::{cpi_transfer, pay_protocol_fee},
    state::{Operation, ProjectConfig, ProtocolConfig},
};
use anchor_lang::prelude::*;

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
            protocol_config.key().as_ref(),
        ],
        bump,
    )]
    pub project_config: Account<'info, ProjectConfig>,
    #[account(
        mut,
        seeds = [b"treasury", project_config.key().as_ref()],
        bump,
    )]
    pub treasury: SystemAccount<'info>,
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
        bumps: &CreateProjectBumps,
    ) -> Result<()> {
        pay_protocol_fee(
            &self.owner,
            &self.protocol_config,
            &self.system_program,
            Operation::CreateProject,
            None,
        )?;

        let rent_exempt = Rent::get()?.minimum_balance(self.treasury.to_account_info().data_len());

        cpi_transfer(
            self.owner.to_account_info(),
            self.treasury.to_account_info(),
            rent_exempt,
            &self.system_program,
        )?;

        self.project_config.set_inner(ProjectConfig {
            protocol_config: self.protocol_config.key(),
            project_id,
            owner: self.owner.key(),
            name,
            description,
            autthorities: authorities,
            treasury_bump: bumps.treasury,
            bump: bumps.project_config,
        });

        Ok(())
    }
}
