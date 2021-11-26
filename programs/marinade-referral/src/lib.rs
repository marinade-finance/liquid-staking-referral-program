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

    ///register refer code to referral program
    pub fn initialize(
        ctx: Context<Initialize>,
        ref_code: String,
        referral_bump: u8,
        beneficiary_bump: u8,
    ) -> ProgramResult {
        process_initialize(ctx, ref_code, referral_bump, beneficiary_bump)
    }

    ///update referral emergency pause
    pub fn pause(ctx: Context<Pause>, pause: bool) -> ProgramResult {
        process_pause(ctx, pause)
    }
}
