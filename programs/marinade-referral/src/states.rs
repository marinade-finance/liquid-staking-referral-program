use anchor_lang::{prelude::*, solana_program::stake::state::StakeState};

use crate::fees::*;

//-----------------------------------------------------
#[account]
pub struct ReferralState {
    // Partner name (string)
    partner_name: String,

    // Beneficiary account (mSOL address)
    beneficiary_account: Pubkey,

    // Transfer-periodicity-seconds (u32 amount of seconds, default: a month)
    transfer_duration: u32,
    // Last transfer to partner timestamp (u64, unix timestamp)
    last_transfer_time: u64,

    // accumulated deposit-sol amount (SOL, u64)
    deposit_sol_amount: u64,
    // accumulated deposit-stake-account amount (SOL, u64)
    depsoit_stake_account_amount: u64,

    // accumulated liquid-unstake amount (SOL, u64)
    liquid_unstake_amount: u64,
    // accumulated count of unstake operations (u64, for stats/monitoring)
    liquid_unstake_operations: u64,

    // accumulated delayed-unstake amount (SOL, u64)
    delayed_unstake_amount: u64,
    // accumulated count of delayed-unstake operations (u64, for stats/monitoring)
    delayed_unstake_operations: u64,

    // Base % cut for the partner (Fee struct, basis points, default 10%)
    base_fee: Fee,
    // Max % cut for the partner (Fee struct, basis points, default 100%)
    max_fee: Fee,
    // Net Stake target for the max % (for example 100K SOL)
    max_net_stake_amount: u64,

    // emergency-pause flag (bool)
    pause: bool,
}

//-----------------------------------------------------
#[account]
pub struct StakeWrapper {
    pub inner: StakeState,
}
