use anchor_lang::prelude::*;

#[error]
pub enum ReferralError {
    #[msg("Access denied")]
    AccessDenied,

    #[msg("Paused")]
    Paused,

    #[msg("Transfer is not available")]
    TransferNotAvailable,

    #[msg("Invalid beneficiary account")]
    InvalidBeneficiaryAccount,
}
