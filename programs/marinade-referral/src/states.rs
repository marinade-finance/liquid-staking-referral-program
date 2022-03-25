use crate::constant::MSOL_TREASURY_AUTH_SEED;
use anchor_lang::prelude::*;
use marinade_finance::{calc::proportional, error::CommonError, Fee};

//-----------------------------------------------------
///marinade-referral-program PDA
#[account]
pub struct GlobalState {
    // Authority (admin address)
    pub admin_account: Pubkey,

    // treasury_msol_account to the referral program - must be fed externally
    pub treasury_msol_account: Pubkey,
    // bump seed for treasury_msol_account auth pda
    pub treasury_msol_auth_bump: u8,
}
impl GlobalState {
    pub fn get_treasury_auth(&self) -> Pubkey {
        Pubkey::create_program_address(
            &[
                &MSOL_TREASURY_AUTH_SEED[..],
                &[self.treasury_msol_auth_bump],
            ],
            &crate::ID,
        )
        .unwrap()
    }
}

//-----------------------------------------------------
///referral PDA
#[account]
pub struct ReferralState {
    // Partner name
    pub partner_name: String, //max-length 20 bytes

    // partner Beneficiary account (native account)
    pub partner_account: Pubkey,
    // token account where to make payment (ATA mSOL address for partner_account)
    pub token_partner_account: Pubkey,

    // Transfer-periodicity-seconds (u32 amount of seconds, default: a month)
    pub transfer_duration: u32,
    // Last transfer to partner timestamp (u64, unix timestamp)
    pub last_transfer_time: i64,

    // accumulated deposit-sol amount (SOL, u64)
    pub deposit_sol_amount: u64,
    // accumulated count of deposit-sol operations (u64, for stats/monitoring)
    pub deposit_sol_operations: u64,

    // accumulated deposit-stake-account amount (SOL, u64)
    pub deposit_stake_account_amount: u64,
    // accumulated count of deposit-stake-account operations (u64, for stats/monitoring)
    pub deposit_stake_account_operations: u64,

    // accumulated liquid-unstake treasury fees (mSOL, u64)
    pub liq_unstake_msol_fees: u64,
    // accumulated liquid-unstake amount (SOL, u64)
    pub liq_unstake_sol_amount: u64,
    // accumulated liquid-unstake amount (mSOL, u64)
    pub liq_unstake_msol_amount: u64,
    // accumulated count of unstake operations (u64, for stats/monitoring)
    pub liq_unstake_operations: u64,

    // accumulated delayed-unstake amount (mSOL, u64)
    pub delayed_unstake_amount: u64,
    // accumulated count of delayed-unstake operations (u64, for stats/monitoring)
    pub delayed_unstake_operations: u64,

    // Base % cut for the partner (basis points, default 1000 => 10%)
    pub base_fee: u32,
    // Max % cut for the partner (basis points, default 1000 => 10%)
    pub max_fee: u32,
    // Net Stake target for the max % (for example 100K SOL)
    pub max_net_stake: u64,

    // emergency-pause flag (bool)
    pub pause: bool,
}

impl ReferralState {
    pub fn reset_accumulators(&mut self) {
        self.deposit_sol_amount = 0;
        self.deposit_sol_operations = 0;

        self.deposit_stake_account_amount = 0;
        self.deposit_stake_account_operations = 0;

        self.liq_unstake_msol_fees = 0;
        self.liq_unstake_msol_amount = 0;
        self.liq_unstake_sol_amount = 0;
        self.liq_unstake_operations = 0;
    }

    pub fn get_liq_unstake_share_amount(&self) -> Result<u64, CommonError> {
        let total_deposit = self.deposit_sol_amount + self.deposit_stake_account_amount;
        // zero if more unstaked than deposited
        let net_stake = total_deposit.saturating_sub(self.liq_unstake_sol_amount);
        let share_fee_bp = if net_stake == 0 {
            self.base_fee // minimum
        } else if net_stake > self.max_net_stake {
            self.max_fee // max
        } else {
            let delta = self.max_fee - self.base_fee;
            // base + delta proportional to net_stake/self.max_net_stake
            self.base_fee + proportional(delta as u64, net_stake, self.max_net_stake)? as u32
        };

        let share_fee = Fee {
            basis_points: share_fee_bp,
        };

        // apply fee basis_points, 100=1%
        Ok(share_fee.apply(self.liq_unstake_msol_fees))
    }
}

//-----------------------------------------------------
