use litesvm::LiteSVM;
use sol_mind_protocol_client::accounts::{ProjectConfig, ProtocolConfig};
use sol_mind_protocol_client::token_manager::accounts::MinterConfig;
use solana_pubkey::Pubkey;

pub struct AccountHelper;

impl AccountHelper {
    pub fn find_protocol_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(&[b"sol-mind-protocol"], program_id).unwrap()
    }

    pub fn get_protocol_config(svm: &LiteSVM, program_id: &Pubkey) -> ProtocolConfig {
        let addr = Self::find_protocol_config_pda(program_id).0;

        let account = svm
            .get_account(&addr)
            .expect("Protocol config account not found");

        ProtocolConfig::from_bytes(&account.data)
            .expect("Failed to deserialize protocol config account")
    }

    pub fn find_project_pda(program_id: &Pubkey, owner: &Pubkey, project_id: u64) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[b"project", owner.as_ref(), &project_id.to_le_bytes()],
            program_id,
        )
        .unwrap()
    }

    pub fn get_project_config(
        svm: &LiteSVM,
        program_id: &Pubkey,
        owner: &Pubkey,
        project_id: u64,
    ) -> ProjectConfig {
        let addr = Self::find_project_pda(program_id, owner, project_id).0;

        let account = svm
            .get_account(&addr)
            .expect("Project config account not found");

        ProjectConfig::from_bytes(&account.data)
            .expect("Failed to deserialize project config account")
    }

    pub fn find_treasury_pda(program_id: &Pubkey, project_config_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(&[b"treasury", project_config_pda.as_ref()], program_id)
            .unwrap()
    }

    pub fn find_minter_config_pda(
        token_manager_program_id: &Pubkey,
        project_config_pda: &Pubkey,
        name: &str,
    ) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[
                b"minter_config",
                project_config_pda.as_ref(),
                name.as_bytes(),
            ],
            token_manager_program_id,
        )
        .unwrap()
    }

    pub fn get_minter_config(
        svm: &LiteSVM,
        token_manager_program_id: &Pubkey,
        project_config_pda: &Pubkey,
        name: &str,
    ) -> MinterConfig {
        let addr =
            Self::find_minter_config_pda(token_manager_program_id, project_config_pda, name).0;

        let account = svm
            .get_account(&addr)
            .expect("Minter config account not found");

        MinterConfig::from_bytes(&account.data)
            .expect("Failed to deserialize minter config account")
    }
}
