use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TradeHub {
    #[max_len(64)]
    pub name: String,
    pub fee_bps: u64,
    pub bump: u8,
}

impl TradeHub {
    pub fn calculate_fee_amount(&self, price: u64) -> u64 {
        price * self.fee_bps / 10000
    }
}
