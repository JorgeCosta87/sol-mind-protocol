use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{helpers::validate_transfer, ProjectConfig};

#[derive(Accounts)]
pub struct TransferProjectFees<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub to: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"project",
            owner.key.as_ref(),
            project_config.protocol_config.as_ref(),
            project_config.project_id.to_le_bytes().as_ref(),
        ],
        bump = project_config.bump,
    )]
    pub project_config: Account<'info, ProjectConfig>,
    #[account(
        mut,
        seeds = [b"treasury", project_config.key().as_ref()],
        bump = project_config.treasury_bump
    )]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> TransferProjectFees<'info> {
    pub fn transfer_project_fees(&mut self, amount: u64) -> Result<()> {
        validate_transfer(&self.treasury.to_account_info(), amount)?;

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.to.to_account_info(),
        };

        let seeds = &[
            b"treasury".as_ref(),
            self.project_config.to_account_info().key.as_ref(),
            &[self.project_config.treasury_bump],
        ];

        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}
