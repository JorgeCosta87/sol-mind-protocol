use anchor_lang::prelude::*;

use crate::errors::ProtocolError;
use crate::state::{Fee, FeesStructure, Operation, ProtocolConfig};

#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"sol-mind-protocol"],
        bump = protocol_config.bump,
        constraint = protocol_config.check_admins(&admin.key()) @ ProtocolError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
}

impl<'info> UpdateFees<'info> {
    pub fn update_fees(&mut self, fees: FeesStructure) -> Result<()> {
        self.protocol_config.fees = fees;
        Ok(())
    }

    pub fn update_single_fee(&mut self, operation: Operation, fee: Fee) -> Result<()> {
        match operation {
            Operation::CreateProject => self.protocol_config.fees.create_project = fee,
            Operation::CreateMinterConfig => self.protocol_config.fees.create_minter_config = fee,
            Operation::CreateTradeHub => self.protocol_config.fees.create_trade_hub = fee,
            Operation::MintAsset => self.protocol_config.fees.mint_asset = fee,
            Operation::TradeNFT => self.protocol_config.fees.trade_nft = fee,
            Operation::Generic => self.protocol_config.fees.generic_operation = fee,
        }
        Ok(())
    }
}
