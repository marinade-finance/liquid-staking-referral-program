use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token;

use crate::{error::*, instructions::*};

pub fn process_transfer_liq_unstake_shares(ctx: Context<TransferLiqShares>) -> ProgramResult {
    let current_time = clock::Clock::get().unwrap().unix_timestamp;
    let elapsed_time = current_time.wrapping_sub(ctx.accounts.referral_state.last_transfer_time);

    if elapsed_time as u32 > ctx.accounts.referral_state.transfer_duration {
        // transfer shared mSOL to partner
        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            ctx.accounts.referral_state.get_liq_unstake_share_amount(),
        )?;

        // sets “Last transfer to partner timestamp“
        ctx.accounts.referral_state.last_transfer_time = current_time;

        // clears all accumulators
        ctx.accounts.referral_state.reset_liq_unstake_accumulators();
    } else {
        return Err(ReferralError::TransferNotAvailable.into());
    }

    Ok(())
}
