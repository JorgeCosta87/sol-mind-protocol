use anchor_lang::prelude::*;

use crate::errors::ProtocolError;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct Fee {
    pub amount: u64,
    pub fee_type: FeeType,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub enum FeeType {
    #[default]
    Fixed,
    Percentage,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct FeesStructure {
    pub create_project: Fee,
    pub create_minter_config: Fee,
    pub create_trade_hub: Fee,
    pub trade_nft: Fee,
    pub mint_asset: Fee,
    pub generic_operation: Fee,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum Operation {
    CreateProject,
    CreateMinterConfig,
    CreateTradeHub,
    TradeNFT,
    MintAsset,
    Generic,
}

#[account]
#[derive(InitSpace)]
pub struct ProtocolConfig {
    #[max_len(3)]
    pub admins: Vec<Pubkey>,
    #[max_len(3)]
    pub whitelist_transfer_addrs: Vec<Pubkey>,
    pub fees: FeesStructure,
    pub bump: u8,
}

impl ProtocolConfig {
    pub fn check_admins(&self, key: &Pubkey) -> bool {
        self.admins.contains(key)
    }

    pub fn check_whitelist_transfer_addrs(&self, key: &Pubkey) -> bool {
        self.whitelist_transfer_addrs.contains(key)
    }

    pub fn get_fee(&self, operation: Operation) -> Fee {
        match operation {
            Operation::CreateProject => self.fees.create_project,
            Operation::CreateMinterConfig => self.fees.create_minter_config,
            Operation::CreateTradeHub => self.fees.create_trade_hub,
            Operation::MintAsset => self.fees.mint_asset,
            Operation::TradeNFT => self.fees.trade_nft,
            Operation::Generic => self.fees.generic_operation,
        }
    }

    pub fn calculate_fee_amount(&self, operation: Operation, base_amount: Option<u64>) -> Result<u64> {
        let fee = self.get_fee(operation);
        match fee.fee_type {
            FeeType::Fixed => Ok(fee.amount),
            FeeType::Percentage => {
                let amount = base_amount.ok_or(error!(ProtocolError::FeeCalculationOverflow))?;
                amount
                    .checked_mul(fee.amount)
                    .ok_or(error!(ProtocolError::FeeCalculationOverflow))?
                    .checked_div(10_000)
                    .ok_or(error!(ProtocolError::FeeCalculationOverflow)) 
            }
        }
    }
}
