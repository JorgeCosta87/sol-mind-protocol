use anchor_lang::prelude::*;

use crate::state::ProjectConfig;

#[derive(Accounts)]
#[instruction(project_id: u64)]
pub struct InitializeProject<'info> {
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

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeProject<'info> {
    pub fn initialize_project(
        &mut self,
        project_id: u64,
        name: String,
        description: String,
        treasury: Pubkey,
        authorities: Vec<Pubkey>,
        bump: u8,
    ) -> Result<()> {
        self.project_config.set_inner(ProjectConfig {
            project_id,
            owner: self.owner.key(),
            name,
            description,
            treasury,
            minter_config_counter: 0,
            autthorities: authorities,
            bump,
        });
        Ok(())
    }
}
