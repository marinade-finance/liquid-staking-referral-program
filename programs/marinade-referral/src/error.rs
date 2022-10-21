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
    ReferralOperationFeeOverMax, // 307

    #[msg("Not allowed for stake-as-collateral-partners")]
    NotAllowedForStakeAsCollateralPartner,
    #[msg("Keep_pct out of valid range")]
    KeepPctOutOfRange,
    #[msg("Max Keep_pct out of valid range, cannot go over 100%")]
    MaxKeepPctOutOfRange,
    #[msg("Min Keep-pct is bounded by value of Max Keep_pct")]
    MinMaxKeepPctOutOfRange,
    #[msg("Stake-account must be delegated to partner validator")]
    StakeAccountMustBeDelegatedToPartnerValidator,
    #[msg("Stake-account authority must be partner account")]
    StakeAccountAuthMustBePartnerAccount,
    #[msg("Only allowed for stake-as-collateral-partners")]
    OnlyAllowedForStakeAsCollateralPartner,
    #[msg("Invalid Global Account")]
    InvalidGlobalAccount,
}
