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

    declare_id!("MR2LqxoSbw831bNy68utpu5n4YqBH3AzDmddkgk9LQv");

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
    pub fn initialize(
        ctx: Context<Initialize>,
        min_keep_pct: u8,
        max_keep_pct: u8,
    ) -> ProgramResult {
        ctx.accounts.process(min_keep_pct, max_keep_pct)
    }

    ///create referral state
    pub fn init_referral_account(
        ctx: Context<InitReferralAccount>,
        partner_name: String,
        validator_vote_key: Option<Pubkey>,
        keep_self_stake_pct: u8,
    ) -> ProgramResult {
        ctx.accounts
            .process(partner_name, validator_vote_key, keep_self_stake_pct)
    }

    ///update referral state
    pub fn update_referral(ctx: Context<UpdateReferral>, pause: bool) -> ProgramResult {
        ctx.accounts.process(pause)
    }

    ///update referral operation fees
    pub fn update_operation_fees(
        ctx: Context<UpdateOperationFees>,
        operation_deposit_sol_fee: Option<u8>,
        operation_deposit_stake_account_fee: Option<u8>,
        operation_liquid_unstake_fee: Option<u8>,
        operation_delayed_unstake_fee: Option<u8>,
    ) -> ProgramResult {
        ctx.accounts.process(
            operation_deposit_sol_fee,
            operation_deposit_stake_account_fee,
            operation_liquid_unstake_fee,
            operation_delayed_unstake_fee,
        )
    }

    /// update partner, authority and beneficiary account based on the new partner
    pub fn change_authority(ctx: Context<ChangeAuthority>) -> ProgramResult {
        ctx.accounts.process()
    }

    ///deposit SOL
    pub fn admin_recognize_deposit(
        ctx: Context<AdminRecognizeDeposit>,
        lamports: u64,
    ) -> ProgramResult {
        ctx.accounts.process(lamports)
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
