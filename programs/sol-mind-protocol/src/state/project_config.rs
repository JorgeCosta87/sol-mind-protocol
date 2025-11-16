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
    pub treasury: Pubkey,
    #[max_len(3)]
    pub autthorities: Vec<Pubkey>,
    pub minter_config_counter: u64,
    pub bump: u8,
}

impl ProjectConfig {
    pub fn check_authorities(&self, key: &Pubkey) -> bool {
        self.autthorities.contains(key)
    }

    pub fn minter_config_next(&self) -> u64 {
        self.minter_config_counter.checked_add(1).unwrap()
    }
}
