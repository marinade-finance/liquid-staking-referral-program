#![allow(unused_imports)]

use anchor_lang::{prelude::*, solana_program::stake::state::StakeState};

use crate::fees::*;

//-----------------------------------------------------
#[account]
pub struct ReferralState {
    // Partner account (partner address)
    pub partner_account: Pubkey,

    // Beneficiary account (mSOL address)
    pub beneficiary_account: Pubkey,

    // Transfer-periodicity-seconds (u32 amount of seconds, default: a month)
    pub transfer_duration: u32,
    // Last transfer to partner timestamp (u64, unix timestamp)
    pub last_transfer_time: i64,

    // accumulated deposit-sol amount (SOL, u64)
    pub deposit_sol_amount: u64,
    // accumulated deposit-stake-account amount (SOL, u64)
    pub depsoit_stake_account_amount: u64,

    // accumulated liquid-unstake amount (SOL, u64)
    pub liq_unstake_amount: u64,
    // accumulated count of unstake operations (u64, for stats/monitoring)
    pub liq_unstake_operations: u64,

    // accumulated delayed-unstake amount (SOL, u64)
    pub del_unstake_amount: u64,
    // accumulated count of delayed-unstake operations (u64, for stats/monitoring)
    pub del_unstake_operations: u64,

    // Base % cut for the partner (Fee struct, basis points, default 10%)
    pub base_fee: Fee,
    // Max % cut for the partner (Fee struct, basis points, default 100%)
    pub max_fee: Fee,
    // Net Stake target for the max % (for example 100K SOL)
    pub max_net_stake_amount: u64,

    // emergency-pause flag (bool)
    pub pause: bool,
}

impl ReferralState {
}

//-----------------------------------------------------
#[account]
pub struct StakeWrapper {
    // TODO: https://github.com/project-serum/anchor/issues/1065
    // pub inner: StakeState,
}
