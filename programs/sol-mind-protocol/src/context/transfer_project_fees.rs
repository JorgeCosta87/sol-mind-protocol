use anchor_lang::prelude::*;

use crate::ProjectConfig;
use crate::errors::ProtocolError;

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
            project_config.project_id.to_le_bytes().as_ref(),
        ],
        bump = project_config.bump,
    )]
    pub project_config: Account<'info, ProjectConfig>,
}

impl<'info> TransferProjectFees<'info> {
    pub fn transfer_project_fees(&mut self, amount: u64) -> Result<()> {
        let project_config_info = self.project_config.to_account_info();
        let current_balance = project_config_info.lamports();
        let rent_exempt = Rent::get()?.minimum_balance(project_config_info.data_len());

        let remaining_balance = current_balance
            .checked_sub(amount)
            .ok_or(ProtocolError::InsufficientFunds)?;
        
        require!(
            remaining_balance >= rent_exempt,
            ProtocolError::MinimumBalanceRequired
        );

        self.project_config.sub_lamports(amount)?;
        self.to.add_lamports(amount)?;

        Ok(())
    }
}
