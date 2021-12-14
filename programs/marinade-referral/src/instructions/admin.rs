use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::token::{TokenAccount, Transfer};
use std::str::FromStr;

use crate::constant::*;
use crate::error::*;
use crate::states::{GlobalState, ReferralState};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    #[account(
        zero,
        address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // mSOL treasury account for referral program
    #[account()]
    pub treasury_msol_account: CpiAccount<'info, TokenAccount>,

    pub system_program: AccountInfo<'info>,
}
impl<'info> Initialize<'info> {
    pub fn process(&mut self, treasury_msol_bump_seed: u8) -> ProgramResult {
        self.global_state.admin_account = *self.admin_account.key;
        self.global_state.treasury_msol_bump_seed = treasury_msol_bump_seed;
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct InitReferralAccount<'info> {
    // global state
    #[account(
        has_one = admin_account,
        address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account, signer
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // partner account
    pub partner_account: AccountInfo<'info>,

    // partner beneficiary mSOL ATA
    #[account()]
    pub token_partner_account: CpiAccount<'info, TokenAccount>,

    #[account(zero)] // must be created but empty, ready to be initialized
    pub referral_state: ProgramAccount<'info, ReferralState>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

impl<'info> InitReferralAccount<'info> {
    pub fn process(&mut self, partner_name: String) -> ProgramResult {
        msg!("process_init_referral_account");
        if partner_name.len() > 20 {
            msg!("max partner_name.len() is 20");
            return Err(ReferralError::PartnerNameTooLong.into());
        }

        // check if beneficiary account address matches to partner_address and msol_mint
        if self.token_partner_account.owner != *self.partner_account.key {
            return Err(ReferralError::InvalidBeneficiaryAccountOwner.into());
        }
        if self.token_partner_account.mint != Pubkey::from_str(MSOL_MINT_ADDRESS).unwrap() {
            return Err(ReferralError::InvalidBeneficiaryAccountMint.into());
        }

        self.referral_state.partner_name = partner_name.clone();

        self.referral_state.partner_account = self.partner_account.key();
        self.referral_state.token_partner_account = self.token_partner_account.key();

        self.referral_state.transfer_duration = DEFAULT_TRANSFER_DURATION;
        self.referral_state.last_transfer_time = clock::Clock::get().unwrap().unix_timestamp;

        self.referral_state.deposit_sol_amount = 0;
        self.referral_state.deposit_sol_operations = 0;

        self.referral_state.deposit_stake_account_amount = 0;
        self.referral_state.deposit_stake_account_operations = 0;

        self.referral_state.liq_unstake_msol_amount = 0;
        self.referral_state.liq_unstake_operations = 0;
        self.referral_state.liq_unstake_msol_fees = 0;

        self.referral_state.delayed_unstake_amount = 0;
        self.referral_state.delayed_unstake_operations = 0;

        self.referral_state.max_net_stake = DEFAULT_MAX_NET_STAKE;
        self.referral_state.base_fee = DEFAULT_BASE_FEE_POINTS;
        self.referral_state.max_fee = DEFAULT_MAX_FEE_POINTS;

        self.referral_state.pause = false;

        Ok(())
    }
}

//--------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    // global state
    #[account(
        mut,
        has_one = admin_account,
        address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // current admin account (must match the one in GlobalState)
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // new admin account
    pub new_admin_account: AccountInfo<'info>,
}
impl<'info> ChangeAuthority<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.global_state.admin_account = *self.new_admin_account.key;
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateReferral<'info> {
    // global state
    #[account(
        has_one = admin_account,
        address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}
impl<'info> UpdateReferral<'info> {
    pub fn process(&mut self, transfer_duration: u32, pause: bool) -> ProgramResult {
        self.referral_state.transfer_duration = transfer_duration;
        self.referral_state.pause = pause;
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct TransferLiqUnstakeShares<'info> {
    // mSOL beneficiary account
    #[account(mut)]
    pub token_partner_account: CpiAccount<'info, TokenAccount>,

    // mSOL treasury token account
    #[account(mut)]
    pub treasury_msol_account: CpiAccount<'info, TokenAccount>,

    // admin
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(
        mut,
        constraint = !referral_state.pause,
        constraint = referral_state.liq_unstake_msol_amount > 0,
        constraint = referral_state.token_partner_account.key() == *token_partner_account.to_account_info().key,
    )]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // global state
    #[account(has_one = admin_account)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    pub token_program: AccountInfo<'info>,
}

impl<'info> TransferLiqUnstakeShares<'info> {
    pub fn process(&mut self) -> ProgramResult {
        let current_time = clock::Clock::get().unwrap().unix_timestamp;
        let elapsed_time = current_time - self.referral_state.last_transfer_time;
        assert!(elapsed_time > 0 && elapsed_time < u32::MAX as i64);

        if elapsed_time as u32 > self.referral_state.transfer_duration {
            // mSOL treasury account seeds
            let authority_seeds = &[
                &MSOL_TREASURY_SEED[..],
                &[self.global_state.treasury_msol_bump_seed],
            ];

            // transfer shared mSOL to partner
            anchor_spl::token::transfer(
                self.into_transfer_to_pda_context()
                    .with_signer(&[&authority_seeds[..]]),
                self.referral_state.get_liq_unstake_share_amount()?,
            )?;

            // sets “Last transfer to partner timestamp“
            self.referral_state.last_transfer_time = current_time;

            // clears all accumulators
            self.referral_state.reset_accumulators();
        } else {
            return Err(ReferralError::TransferNotAvailable.into());
        }

        Ok(())
    }

    pub fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.treasury_msol_account.to_account_info(),
            to: self.token_partner_account.to_account_info(),
            authority: self.treasury_msol_account.to_account_info(),
        };

        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}
