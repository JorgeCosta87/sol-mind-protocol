use litesvm::LiteSVM;
use mpl_core::{Asset, Collection};
use solana_pubkey::Pubkey;

pub struct MplUtils;

impl MplUtils {
    pub fn get_collection(svm: &LiteSVM, mint: &Pubkey) -> Collection {
        let account = svm
            .get_account(mint)
            .expect("Collection Asset account not found");

        *Collection::deserialize(&account.data).expect("Failed to deserialize collection account")
    }

    pub fn get_asset(svm: &LiteSVM, mint: &Pubkey) -> Asset {
        let account = svm.get_account(mint).expect("Asset account not found");

        *Asset::deserialize(&account.data).expect("Failed to deserialize asset account")
    }
}
