use anchor_lang::error_code;

#[error_code]
pub enum ErrorCode {
    #[msg("Collection requires an uri")]
    RequiredUri,
    #[msg("PluginAuthorityPair could not be deserialized")]
    InvalidPlugin,
    #[msg("If minter config don't have and asset config provide a name and uri")]
    RequireNameAnddUri,
    #[msg("Mismatch with the collection on minter config")]
    CollectionMismatch,
    #[msg("Max supply reached")]
    MaxSupplyReached,
}
