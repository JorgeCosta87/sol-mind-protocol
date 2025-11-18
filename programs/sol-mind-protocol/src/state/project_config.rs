use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProjectConfig {
    pub project_id: u64,
    pub owner: Pubkey,
    #[max_len(64)]
    pub name: String,
    #[max_len(200)]
    pub description: String,
    #[max_len(3)]
    pub autthorities: Vec<Pubkey>,
    pub treasury_bump: u8,
    pub bump: u8,
}

impl ProjectConfig {
    pub fn check_authorities(&self, key: &Pubkey) -> bool {
        self.autthorities.contains(key)
    }
}
