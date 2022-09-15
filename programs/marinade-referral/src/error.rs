use anchor_lang::prelude::*;

#[error]
pub enum ReferralError {
    #[msg("Access denied")]
    AccessDenied, // 300

    #[msg("Paused")]
    Paused, // 301

    #[msg("Transfer is not available yet")]
    TransferNotAvailable, // 302

    #[msg("Invalid partner account owner")]
    InvalidPartnerAccountOwner, // 303
    #[msg("Invalid partner account mint")]
    InvalidPartnerAccountMint, // 304
    #[msg("Partner name too long")]
    PartnerNameTooLong, // 305
    #[msg("Mint account is not initialized")]
    NotInitializedMintAccount, // 306

    #[msg("Referral operation fee was set over the maximum permitted amount")]
    ReferralOperationFeeOverMax,
}
