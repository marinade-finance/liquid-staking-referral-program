use anchor_lang::prelude::*;

#[error]
pub enum ReferralError {
    #[msg("Access denied")]
    AccessDenied,

    #[msg("Paused")]
    Paused,

    #[msg("Claim is not available")]
    ClaimNotAvailable,
}
