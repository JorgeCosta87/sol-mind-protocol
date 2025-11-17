use anchor_lang::prelude::*;

use crate::errors::ProtocolError;
use crate::state::ProtocolConfig;

#[derive(Accounts)]
pub struct ProtocolFeesTransfer<'info> {
    #[account(
        mut,
        constraint = protocol_config.check_admins(&admin.key()) @ ProtocolError::Unauthorized
    )]
    pub admin: Signer<'info>,
    #[account(
        mut,
        constraint = protocol_config.check_whitelist_transfer_addrs(&to.key()) @ ProtocolError::AddressNotWhiteListed
    )]
    pub to: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"sol-mind-protocol"],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

impl<'info> ProtocolFeesTransfer<'info> {
    pub fn transfer_protocol_fees(&mut self, amount: u64) -> Result<()> {
        let protocol_config_info = self.protocol_config.to_account_info();
        let current_balance = protocol_config_info.lamports();
        let rent_exempt = Rent::get()?.minimum_balance(protocol_config_info.data_len());

        let remaining_balance = current_balance
            .checked_sub(amount)
            .ok_or(ProtocolError::InsufficientFunds)?;
        
        require!(
            remaining_balance >= rent_exempt,
            ProtocolError::MinimumBalanceRequired
        );

        self.protocol_config.sub_lamports(amount)?;
        self.to.add_lamports(amount)?;

        Ok(())
    }
}
