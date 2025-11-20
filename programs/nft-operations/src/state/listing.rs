use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing {
    #[max_len(64)]
    pub asset: String,
    pub price: u64,
    pub mints_counter: u64,
    pub created_at: u64,
    pub collection: Option<Pubkey>,
    pub bump: u8,
}
