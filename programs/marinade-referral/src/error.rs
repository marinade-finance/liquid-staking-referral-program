use anchor_lang::prelude::*;

#[error]
pub enum ReferralError {
    #[msg("Access denied")]
    AccessDenied, // 300

    #[msg("Paused")]
    Paused, // 301

    #[msg("Transfer is not available")]
    TransferNotAvailable, // 302

    #[msg("Invalid beneficiary account owner")]
    InvalidBeneficiaryAccountOwner, // 303
    #[msg("Invalid beneficiary account mint")]
    InvalidBeneficiaryAccountMint, // 304
    #[msg("Partner name too long")]
    PartnerNameTooLong, // 305
}
