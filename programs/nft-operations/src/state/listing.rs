use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub owner: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub created_at: i64,
    pub bump: u8,
}
