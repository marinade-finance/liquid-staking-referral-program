use anchor_lang::prelude::*;

///associated token
pub mod associated_token;
///constant
pub mod constant;
///cpi context instructions
pub mod cpi_context_instructions;
///error
pub mod error;
///fees
pub mod fees;
///instructions
pub mod instructions;
///processor
pub mod processor;
///states
pub mod states;
///utils
pub mod utils;

use crate::{instructions::*, processor::*};

#[program]
pub mod marinade_referral {
    use super::*;

    ///create global state
    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> ProgramResult {
        process_initialize(ctx, bump)
    }

    ///create referral state
    pub fn create_referral_pda(
        ctx: Context<CreateReferralPda>,
        bump: u8,
        partner_name: [u8; 10],
    ) -> ProgramResult {
        process_create_referral_pda(ctx, bump, partner_name)
    }

    ///update referral state
    pub fn update_referral(
        ctx: Context<UpdateReferral>,
        transfer_duration: u32,
        pause: bool,
    ) -> ProgramResult {
        process_update_referral(ctx, transfer_duration, pause)
    }

    ///update partner, authority and beneficiary account based on the new partner
    pub fn change_authority(ctx: Context<ChangeAuthority>) -> ProgramResult {
        process_change_authority(ctx)
    }

    ///deposit SOL
    pub fn deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
        process_deposit(ctx, lamports)
    }

    ///deposit stake account
    pub fn deposit_stake_account(
        ctx: Context<DepositStakeAccount>,
        validator_index: u32,
    ) -> ProgramResult {
        process_deposit_stake_account(ctx, validator_index)
    }

    ///liquid-unstake mSOL
    pub fn liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> ProgramResult {
        process_liquid_unstake(ctx, msol_amount)
    }

    ///transfer shares, treasury holders can transfer shares manually
    pub fn transfer_liq_shares(ctx: Context<TransferLiqShares>) -> ProgramResult {
        process_transfer_liq_shares(ctx)
    }
}
