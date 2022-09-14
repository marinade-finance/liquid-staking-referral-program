use anchor_lang::prelude::*;
use marinade_finance::{calc::proportional, error::CommonError, Fee};

//-----------------------------------------------------
///marinade-referral-program PDA
#[account]
pub struct GlobalState {
    // Authority (admin address)
    pub admin_account: Pubkey,

    // msol mint account to verify the mint of partner msol account (must be fed externally)
    pub msol_mint_account: Pubkey,
}

//-----------------------------------------------------
///referral PDA
#[account]
pub struct ReferralState {
    // Partner name
    pub partner_name: String, //max-length 20 bytes

    // partner Beneficiary account (native account)
    pub partner_account: Pubkey,
    // token account where to make payment (mSOL address for partner_account)
    pub msol_token_partner_account: Pubkey,

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

    // fees that will be assigned to referrals per operation, calculated in basis points
    pub operation_deposit_sol_fee: Fee,
    pub operation_deposit_stake_account_fee: Fee,
    pub operation_liquid_unstake_fee: Fee,
    pub operation_delayed_unstake_fee: Fee,
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
