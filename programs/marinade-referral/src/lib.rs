use anchor_lang::prelude::*;
use anchor_lang::solana_program::declare_id;
use anchor_lang::solana_program::pubkey::Pubkey;

use instructions::{admin::*, deposit_sol::*, deposit_stake_account::*, liquid_unstake::*};

///constant
pub mod constant;
///error
pub mod error;
///instructions
pub mod instructions;
///states
pub mod states;

#[program]
pub mod marinade_referral {
    use super::*;

    declare_id!("mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY");

    ///deposit SOL
    pub fn deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
        ctx.accounts.process(lamports)
    }

    ///deposit stake account
    pub fn deposit_stake_account(
        ctx: Context<DepositStakeAccount>,
        validator_index: u32,
    ) -> ProgramResult {
        ctx.accounts.process(validator_index)
    }

    ///liquid-unstake mSOL
    pub fn liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> ProgramResult {
        ctx.accounts.process(msol_amount)
    }

    ///Admin
    ///create global state
    pub fn initialize(ctx: Context<Initialize>, treasury_msol_auth_bump: u8) -> ProgramResult {
        ctx.accounts.process(treasury_msol_auth_bump)
    }

    ///create referral state
    pub fn init_referral_account(
        ctx: Context<InitReferralAccount>,
        partner_name: String,
    ) -> ProgramResult {
        ctx.accounts.process(partner_name)
    }

    ///update referral state
    pub fn update_referral(
        ctx: Context<UpdateReferral>,
        transfer_duration: u32,
        pause: bool,
    ) -> ProgramResult {
        ctx.accounts.process(transfer_duration, pause)
    }

    ///update partner, authority and beneficiary account based on the new partner
    pub fn change_authority(ctx: Context<ChangeAuthority>) -> ProgramResult {
        ctx.accounts.process()
    }

    pub fn delete_program_account(ctx: Context<DeleteProgramAccount>) -> ProgramResult {
        ctx.accounts.process()
    }
    // required for https://docs.rs/solana-program-test/1.7.11/solana_program_test/index.html
    // in order to load two programs with entry points into the simulator
    pub fn test_entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        if data.len() < 8 {
            return Err(anchor_lang::__private::ErrorCode::InstructionMissing.into());
        }
        dispatch(program_id, accounts, data).map_err(|e| {
            ::solana_program::log::sol_log(&e.to_string());
            e
        })
    }
}
