use anchor_lang::error_code;

#[error_code]
pub enum ErrorCode {
    #[msg("PluginAuthorityPair could not be deserialized")]
    InvalidPlugin,
}
