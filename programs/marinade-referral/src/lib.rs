use anchor_lang::prelude::*;

/// constant
pub mod constant;
/// error
pub mod error;
/// fees
pub mod fees;
/// instructions
pub mod instructions;
///processor
pub mod processor;
/// states
pub mod states;
/// utils
pub mod utils;

use crate::{instructions::*, processor::*};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod marinade_referral {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        process_initialize(ctx)
    }
}
