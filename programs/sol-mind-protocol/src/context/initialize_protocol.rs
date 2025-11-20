use anchor_lang::prelude::*;

use crate::state::{FeesStructure, ProtocolConfig};

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + ProtocolConfig::INIT_SPACE,
        seeds = [
            b"sol-mind-protocol",
        ],
        bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeProtocol<'info> {
    pub fn initialize_protocol(
        &mut self,
        admins: Vec<Pubkey>,
        whitelist_transfer_addrs: Vec<Pubkey>,
        fees: FeesStructure,
        bump: u8,
    ) -> Result<()> {
        self.protocol_config.set_inner(ProtocolConfig {
            admins,
            whitelist_transfer_addrs,
            fees,
            bump,
        });

        Ok(())
    }
}
