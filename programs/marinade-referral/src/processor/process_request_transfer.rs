use anchor_lang::{prelude::*, solana_program::clock};

use crate::instructions::*;

pub fn process_request_transfer(ctx: Context<RequestTransfer>) -> ProgramResult {
    let current_time = clock::Clock::get().unwrap().unix_timestamp;
    let elapsed_time = current_time.wrapping_sub(ctx.accounts.state.last_transfer_time);

    if elapsed_time as u32 > ctx.accounts.state.transfer_duration {
        // TODO: transfer shared mSOL to partner

        // sets “Last transfer to partner timestamp“
        ctx.accounts.state.last_transfer_time = current_time;

        // clears all accumulators
        ctx.accounts.state.reset_liq_unstake_accumulators();
    }

    Ok(())
}
