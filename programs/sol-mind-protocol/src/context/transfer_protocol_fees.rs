use anchor_lang::prelude::*;

use crate::{errors::ProtocolError, helpers::validate_transfer, state::ProtocolConfig};

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
        validate_transfer(&self.protocol_config.to_account_info(), amount)?;

        self.protocol_config.sub_lamports(amount)?;
        self.to.add_lamports(amount)?;

        Ok(())
    }
}
