use anchor_lang::error_code;

#[error_code]
pub enum ProtocolError {
    #[msg("Unauthorized: Only admins can perform this action")]
    Unauthorized,
    #[msg("Address not whitelist for PDA transfer")]
    AddressNotWhiteListed,
    #[msg("Not enough funds.")]
    InsufficientFunds,
    #[msg("Minimun balance required for rend exempt")]
    MinimumBalanceRequired,
}
