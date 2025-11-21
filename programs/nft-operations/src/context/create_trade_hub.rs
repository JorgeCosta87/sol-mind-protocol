use anchor_lang::prelude::*;

use sol_mind_protocol::helpers::pay_protocol_fee;
use sol_mind_protocol::{Operation, ProjectConfig};

use crate::errors::ErrorCode;
use crate::state::TradeHub;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateTradeHub<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = project_config.check_authorities(authority.key) @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + TradeHub::INIT_SPACE,
        seeds = [
            b"trade_hub",
            name.as_bytes(),
            project_config.key().as_ref(),
        ],
        bump,
    )]
    pub trade_hub: Account<'info, TradeHub>,
    #[account(
        seeds = [
            b"project",
            project_config.owner.as_ref(),
            project_config.project_id.to_le_bytes().as_ref(),
        ],
        bump = project_config.bump,
        seeds::program = sol_mind_protocol::ID,
    )]
    pub project_config: Account<'info, ProjectConfig>,
    #[account(
        mut,
        seeds = [b"sol-mind-protocol"],
        bump = protocol_config.bump,
        seeds::program = sol_mind_protocol::ID,
    )]
    pub protocol_config: Account<'info, sol_mind_protocol::ProtocolConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateTradeHub<'info> {
    pub fn create_trade_hub(&mut self, name: String, fee_bps: u64, bump: u8) -> Result<()> {
        pay_protocol_fee(
            &self.payer,
            &self.protocol_config,
            &self.system_program,
            Operation::CreateTradeHub,
            None,
        )?;

        self.trade_hub.set_inner(TradeHub {
            project: self.project_config.key(),
            name,
            fee_bps: fee_bps,
            bump,
        });
        Ok(())
    }
}
