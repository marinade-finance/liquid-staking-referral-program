use anchor_lang::prelude::*;

use crate::account_structs::*;

pub fn process_update_referral(
    ctx: Context<UpdateReferral>,
    transfer_duration: u32,
    pause: bool,
) -> ProgramResult {
    ctx.accounts.referral_state.transfer_duration = transfer_duration;
    ctx.accounts.referral_state.pause = pause;

    Ok(())
}
