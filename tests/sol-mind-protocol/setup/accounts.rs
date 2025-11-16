use litesvm::LiteSVM;
use mpl_core::{Asset, Collection};
use sol_mind_protocol_client::generated::accounts::{MinterConfig, ProjectConfig};
use solana_pubkey::Pubkey;

pub struct AccountHelper<'a> {
    svm: &'a LiteSVM,
    program_id: Pubkey,
}

impl<'a> AccountHelper<'a> {
    pub fn new(svm: &'a LiteSVM, program_id: Pubkey) -> Self {
        Self { svm, program_id }
    }

    pub fn find_project_pda(&self, owner: Pubkey, project_id: u64) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[b"project", owner.as_ref(), &project_id.to_le_bytes()],
            &self.program_id,
        )
        .unwrap()
    }

    pub fn get_project_config(&self, owner: Pubkey, project_id: u64) -> ProjectConfig {
        let addr = self.find_project_pda(owner, project_id).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("Project config account not found");

        ProjectConfig::from_bytes(&account.data)
            .expect("Failed to deserialize project config account")
    }

    pub fn find_minter_config_pda(&self, owner: Pubkey, project_id: u64) -> (Pubkey, u8) {
        let project_config = self.get_project_config(owner, project_id);
        Pubkey::try_find_program_address(
            &[
                b"minter_config",
                &project_id.to_le_bytes(),
                &project_config.minter_config_counter.to_le_bytes(),
            ],
            &self.program_id,
        )
        .unwrap()
    }

    pub fn find_next_minter_config_pda(&self, owner: Pubkey, project_id: u64) -> (Pubkey, u8) {
        let project_config = self.get_project_config(owner, project_id);
        Pubkey::try_find_program_address(
            &[
                b"minter_config",
                &project_id.to_le_bytes(),
                &(project_config.minter_config_counter + 1).to_le_bytes(),
            ],
            &self.program_id,
        )
        .unwrap()
    }

    pub fn get_minter_config(&self, owner: Pubkey, project_id: u64) -> MinterConfig {
        let addr = self.find_minter_config_pda(owner, project_id).0;

        let account = self
            .svm
            .get_account(&addr)
            .expect("Minter config account not found");

        MinterConfig::from_bytes(&account.data)
            .expect("Failed to deserialize minter config account")
    }

    pub fn get_collection(&self, mint: Pubkey) -> Collection {
        let account = self
            .svm
            .get_account(&mint)
            .expect("Collection Asset account  not found");

        *Collection::deserialize(&account.data).expect("Failed to deserialize asset account")
    }

    pub fn get_asset(&self, mint: Pubkey) -> Asset {
        let account = self
            .svm
            .get_account(&mint)
            .expect("Asset account not found");

        *Asset::deserialize(&account.data).expect("Failed to deserialize asset account")
    }
}
