use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

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
        seeds = [b"sol-mind-protocol"],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(
        mut,
        seeds = [b"treasury", protocol_config.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ProtocolFeesTransfer<'info> {
    pub fn transfer_protocol_fees(
        &mut self,
        amount: u64,
        bumps: &ProtocolFeesTransferBumps,
    ) -> Result<()> {
        validate_transfer(&self.treasury.to_account_info(), amount)?;

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.to.to_account_info(),
        };

        let protocol_key = self.protocol_config.key();
        let seeds = &[b"treasury", protocol_key.as_ref(), &[bumps.treasury]];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}
