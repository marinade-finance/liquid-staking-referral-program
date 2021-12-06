use anchor_lang::prelude::*;
use anchor_lang::solana_program::declare_id;
use anchor_lang::solana_program::pubkey::Pubkey;

///instructions
pub mod account_structs;
///associated token
pub mod associated_token;
///constant
pub mod constant;
///cpi context instructions
pub mod cpi_context_instructions;
///error
pub mod error;
///processor
pub mod processor;
///states
pub mod states;

use crate::process_init_referral_account::process_init_referral_account;
use crate::{account_structs::*, processor::*};

// pub fn test_ep(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8])
//  -> ProgramResult {
//     if data.len() < 8 {
//         return Err(anchor_lang::__private::ErrorCode::InstructionMissing.into());
//     }
//     dispatch(program_id, accounts,
//              data).map_err(|e|
//                                {
//                                    ::solana_program::log::sol_log(&e.to_string());
//                                    e
//                                })
// }

#[program]
pub mod marinade_referral {
    use super::*;

    declare_id!("FqYPYHc3man91xYDCugbGuDdWgkNLp5TvbXPascHW6MR");

    // required for https://docs.rs/solana-program-test/1.7.11/solana_program_test/index.html
    // in order to load two programs with entrypoints into the simulator
    pub fn test_entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        if data.len() < 8 {
            return Err(anchor_lang::__private::ErrorCode::InstructionMissing.into());
        }
        dispatch(program_id, accounts, data).map_err(|e| {
            ::solana_program::log::sol_log(&e.to_string());
            e
        })
    }
    ///create global state
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        process_initialize(ctx)
    }

    ///create referral state
    pub fn init_referral_account(
        ctx: Context<InitReferralAccount>,
        partner_name: String,
    ) -> ProgramResult {
        process_init_referral_account(ctx, partner_name)
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
    pub fn transfer_liq_unstake_shares(ctx: Context<TransferLiqUnstakeShares>) -> ProgramResult {
        process_transfer_liq_unstake_shares(ctx)
    }

    /*
    // Utility to delete state or referral accounts in devnet
    pub fn delete_account(ctx: Context<DeleteAccount>) -> ProgramResult {
        // set lamports to zero
        **ctx.accounts.beneficiary.lamports.borrow_mut() = ctx.accounts.beneficiary.lamports() + ctx.accounts.to_delete.lamports();
        **ctx.accounts.to_delete.lamports.borrow_mut() = 0;
        Ok(())
    }
    */
}
