use anchor_lang::prelude::*;

///constant
pub mod constant;
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

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod marinade_referral {
    use super::*;

    ///create referral state
    pub fn initialize(ctx: Context<Initialize>, partner_name: [u8; 10]) -> ProgramResult {
        process_initialize(ctx, partner_name)
    }

    ///update referral state
    pub fn update(ctx: Context<Update>, transfer_duration: u32, pause: bool) -> ProgramResult {
        process_update(ctx, transfer_duration, pause)
    }

    ///update partner, authority
    pub fn update_authority(ctx: Context<UpdateAuthority>) -> ProgramResult {
        process_update_authority(ctx)
    }

    ///deposit SOL
    pub fn deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
        process_deposit(ctx, lamports)
    }

    ///liquid-unstake mSOL
    pub fn liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> ProgramResult {
        process_liquid_unstake(ctx, msol_amount)
    }
}
