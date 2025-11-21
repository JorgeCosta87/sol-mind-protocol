use litesvm::LiteSVM;
use sol_mind_protocol_client::{
    accounts::{ProjectConfig, ProtocolConfig},
    nft_operations::accounts::MinterConfig,
    nft_operations::accounts::{Listing, TradeHub},
    NFT_OPERATIONS_ID, SOL_MIND_PROTOCOL_ID,
};
use solana_pubkey::Pubkey;

pub struct AccountHelper;

impl AccountHelper {
    pub fn find_protocol_config_pda() -> (Pubkey, u8) {
        Pubkey::try_find_program_address(&[b"sol-mind-protocol"], &SOL_MIND_PROTOCOL_ID).unwrap()
    }

    pub fn get_protocol_config(svm: &LiteSVM) -> ProtocolConfig {
        let addr = Self::find_protocol_config_pda().0;

        let account = svm
            .get_account(&addr)
            .expect("Protocol config account not found");

        ProtocolConfig::from_bytes(&account.data)
            .expect("Failed to deserialize protocol config account")
    }

    pub fn find_project_pda(owner: &Pubkey, project_id: u64) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[b"project", owner.as_ref(), &project_id.to_le_bytes()],
            &SOL_MIND_PROTOCOL_ID,
        )
        .unwrap()
    }

    pub fn get_project_config(svm: &LiteSVM, owner: &Pubkey, project_id: u64) -> ProjectConfig {
        let addr = Self::find_project_pda(owner, project_id).0;

        let account = svm
            .get_account(&addr)
            .expect("Project config account not found");

        ProjectConfig::from_bytes(&account.data)
            .expect("Failed to deserialize project config account")
    }

    pub fn find_treasury_pda(project_config_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[b"treasury", project_config_pda.as_ref()],
            &SOL_MIND_PROTOCOL_ID,
        )
        .unwrap()
    }

    pub fn find_minter_config_pda(project_config_pda: &Pubkey, name: &str) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[
                b"minter_config",
                project_config_pda.as_ref(),
                name.as_bytes(),
            ],
            &NFT_OPERATIONS_ID,
        )
        .unwrap()
    }

    pub fn get_minter_config(
        svm: &LiteSVM,
        project_config_pda: &Pubkey,
        name: &str,
    ) -> MinterConfig {
        let addr = Self::find_minter_config_pda(project_config_pda, name).0;

        let account = svm
            .get_account(&addr)
            .expect("Minter config account not found");

        MinterConfig::from_bytes(&account.data)
            .expect("Failed to deserialize minter config account")
    }

    pub fn find_trade_hub_pda(name: &str, project_config_pda: &Pubkey) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[b"trade_hub", name.as_bytes(), project_config_pda.as_ref()],
            &NFT_OPERATIONS_ID,
        )
        .unwrap()
    }

    pub fn get_trade_hub(svm: &LiteSVM, name: &str, project_config_pda: &Pubkey) -> TradeHub {
        let addr = Self::find_trade_hub_pda(name, project_config_pda).0;

        let account = svm.get_account(&addr).expect("Trade hub account not found");

        TradeHub::from_bytes(&account.data).expect("Failed to deserialize trade hub account")
    }

    pub fn find_listing_pda(asset: &Pubkey, trade_hub: &Pubkey) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[b"listing", asset.as_ref(), trade_hub.as_ref()],
            &NFT_OPERATIONS_ID,
        )
        .unwrap()
    }

    pub fn get_listing(svm: &LiteSVM, asset: &Pubkey, trade_hub: &Pubkey) -> Listing {
        let addr = Self::find_listing_pda(asset, trade_hub).0;

        let account = svm.get_account(&addr).expect("Listing account not found");

        Listing::from_bytes(&account.data).expect("Failed to deserialize listing account")
    }
}
