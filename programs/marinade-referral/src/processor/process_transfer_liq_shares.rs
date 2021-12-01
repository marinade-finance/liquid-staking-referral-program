use anchor_lang::{prelude::*, solana_program::clock};

use crate::{error::*, instructions::*};

pub fn process_transfer_liq_shares(ctx: Context<TransferLiqShares>) -> ProgramResult {
    let current_time = clock::Clock::get().unwrap().unix_timestamp;
    let elapsed_time = current_time.wrapping_sub(ctx.accounts.referral_state.last_transfer_time);

    if elapsed_time as u32 > ctx.accounts.referral_state.transfer_duration {
        // TODO: transfer shared mSOL to partner

        // sets “Last transfer to partner timestamp“
        ctx.accounts.referral_state.last_transfer_time = current_time;

        // clears all accumulators
        ctx.accounts.referral_state.reset_liq_unstake_accumulators();
    } else {
        return Err(ReferralError::TransferNotAvailable.into());
    }

    Ok(())
}
