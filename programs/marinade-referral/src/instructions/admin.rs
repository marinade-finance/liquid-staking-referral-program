use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use solana_program::program_pack::IsInitialized;

use crate::constant::*;
use crate::error::ReferralError::*;
use crate::error::*;
use crate::states::{GlobalState, ReferralState};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    #[account(zero)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    #[account()]
    pub msol_mint_account: CpiAccount<'info, Mint>,

    pub foreman_1: AccountInfo<'info>,
    pub foreman_2: AccountInfo<'info>,
}
impl<'info> Initialize<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.global_state.admin_account = self.admin_account.key();
        self.global_state.msol_mint_account = self.msol_mint_account.key();
        self.global_state.foreman_1 = self.foreman_1.key();
        self.global_state.foreman_2 = self.foreman_2.key();

        self.global_state.max_keep_pct = 80;
        self.global_state.min_keep_pct = 20;

        // verify if the account that should be considered as MSOL mint is an active mint account
        if !self.msol_mint_account.is_initialized() {
            return Err(ReferralError::NotInitializedMintAccount.into());
        }
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct InitReferralAccount<'info> {
    // global state
    #[account(
        constraint = *signer.key == global_state.admin_account || *signer.key == global_state.foreman_1 || *signer.key == global_state.foreman_2
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account or foreman account
    #[account(signer)]
    pub signer: AccountInfo<'info>,

    #[account(zero)] // must be created but empty, ready to be initialized
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // partner main account
    pub partner_account: AccountInfo<'info>,

    // partner mSOL beneficiary token account
    #[account()]
    pub msol_token_partner_account: CpiAccount<'info, TokenAccount>,
}

impl<'info> InitReferralAccount<'info> {
    pub fn process(
        &mut self,
        partner_name: String,
        validator_vote_key: Option<Pubkey>,
        keep_self_stake_pct: u8,
    ) -> ProgramResult {
        msg!("process_init_referral_account");
        if partner_name.len() > 20 {
            msg!("max partner_name.len() is 20");
            return Err(ReferralError::PartnerNameTooLong.into());
        }

        // check if beneficiary account address matches to partner_address and msol_mint
        check_partner_accounts(
            &self.partner_account,
            &self.msol_token_partner_account,
            &self.global_state.msol_mint_account,
        )?;

        self.referral_state.partner_name = partner_name.clone();

        self.referral_state.validator_vote_key = validator_vote_key;
        // if stake-as-collateral mode
        if validator_vote_key.is_some() {
            if !(keep_self_stake_pct >= self.global_state.min_keep_pct
                && keep_self_stake_pct <= self.global_state.max_keep_pct)
            {
                msg!(
                    "keep_pct {} must be >= {} and <= {}",
                    keep_self_stake_pct,
                    self.global_state.min_keep_pct,
                    self.global_state.max_keep_pct
                );
                return Err(KeepPctOutOfRange.into());
            };
            self.referral_state.keep_self_stake_pct = keep_self_stake_pct
        };

        self.referral_state.partner_account = self.partner_account.key();
        self.referral_state.msol_token_partner_account = self.msol_token_partner_account.key();

        self.referral_state.deposit_sol_amount = 0;
        self.referral_state.deposit_sol_operations = 0;

        self.referral_state.deposit_stake_account_amount = 0;
        self.referral_state.deposit_stake_account_operations = 0;

        self.referral_state.liq_unstake_msol_amount = 0;
        self.referral_state.liq_unstake_sol_amount = 0;
        self.referral_state.liq_unstake_operations = 0;
        self.referral_state.liq_unstake_msol_fees = 0;

        self.referral_state.delayed_unstake_amount = 0;
        self.referral_state.delayed_unstake_operations = 0;

        self.referral_state.max_net_stake = DEFAULT_MAX_NET_STAKE;
        self.referral_state.base_fee = DEFAULT_BASE_FEE_POINTS;
        self.referral_state.max_fee = DEFAULT_MAX_FEE_POINTS;

        self.referral_state.pause = false;

        self.referral_state.operation_deposit_sol_fee = DEFAULT_OPERATION_FEE_POINTS;
        self.referral_state.operation_deposit_stake_account_fee = DEFAULT_OPERATION_FEE_POINTS;
        self.referral_state.operation_liquid_unstake_fee = DEFAULT_OPERATION_FEE_POINTS;
        self.referral_state.operation_delayed_unstake_fee = DEFAULT_OPERATION_FEE_POINTS;

        self.referral_state.accum_deposit_sol_fees = 0;
        self.referral_state.accum_deposit_stake_account_fee = 0;
        self.referral_state.accum_liquid_unstake_fee = 0;
        self.referral_state.accum_delayed_unstake_fee = 0;

        Ok(())
    }
}

fn check_partner_accounts<'info>(
    partner_account: &AccountInfo<'info>,
    msol_token_partner_account: &CpiAccount<'info, TokenAccount>,
    msol_mint_pk: &Pubkey,
) -> ProgramResult {
    // check if beneficiary account address matches to partner_address and msol_mint
    if msol_token_partner_account.owner != *partner_account.key {
        msg!(
            "msol token partner account {} has to be owned by partner account {}",
            msol_token_partner_account.key(),
            partner_account.key
        );
        return Err(ReferralError::InvalidPartnerAccountOwner.into());
    }
    if msol_token_partner_account.mint != *msol_mint_pk {
        msg!(
            "mint of msol token partner account {} has to be same as global state mint account {}",
            msol_token_partner_account.key(),
            msol_mint_pk
        );
        return Err(ReferralError::InvalidPartnerAccountMint.into());
    }
    Ok(())
}

//--------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    // global state
    #[account(mut, has_one = admin_account)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // current admin account (must match the one in GlobalState)
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // new admin account
    pub new_admin_account: AccountInfo<'info>,

    // new foremen accounts
    pub new_foreman_1: AccountInfo<'info>,
    pub new_foreman_2: AccountInfo<'info>,
}
impl<'info> ChangeAuthority<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.global_state.admin_account = *self.new_admin_account.key;
        self.global_state.foreman_1 = *self.new_foreman_1.key;
        self.global_state.foreman_2 = *self.new_foreman_2.key;
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateReferral<'info> {
    // global state
    #[account(has_one = admin_account)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // partner main account
    pub new_partner_account: AccountInfo<'info>,

    // partner mSOL beneficiary token account
    #[account()]
    pub new_msol_token_partner_account: CpiAccount<'info, TokenAccount>,
}
impl<'info> UpdateReferral<'info> {
    pub fn process(&mut self, pause: bool) -> ProgramResult {
        self.referral_state.pause = pause;

        if *self.new_partner_account.key != self.referral_state.partner_account
            || self.new_msol_token_partner_account.key()
                != self.referral_state.msol_token_partner_account
        {
            self.referral_state.partner_account = *self.new_partner_account.key;
            self.referral_state.msol_token_partner_account =
                self.new_msol_token_partner_account.key();
            check_partner_accounts(
                &self.new_partner_account,
                &self.new_msol_token_partner_account,
                &self.global_state.msol_mint_account,
            )?;
        }

        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateOperationFees<'info> {
    // global state
    #[account(
        constraint = *signer.key == global_state.admin_account || *signer.key == global_state.foreman_1 || *signer.key == global_state.foreman_2
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin or foreman account
    #[account(signer)]
    pub signer: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}
impl<'info> UpdateOperationFees<'info> {
    pub fn process(
        &mut self,
        operation_deposit_sol_fee: Option<u8>,
        operation_deposit_stake_account_fee: Option<u8>,
        operation_liquid_unstake_fee: Option<u8>,
        operation_delayed_unstake_fee: Option<u8>,
    ) -> ProgramResult {
        set_fee_checked(
            &mut self.referral_state.operation_deposit_sol_fee,
            operation_deposit_sol_fee,
        )?;
        set_fee_checked(
            &mut self.referral_state.operation_deposit_stake_account_fee,
            operation_deposit_stake_account_fee,
        )?;
        set_fee_checked(
            &mut self.referral_state.operation_liquid_unstake_fee,
            operation_liquid_unstake_fee,
        )?;
        set_fee_checked(
            &mut self.referral_state.operation_delayed_unstake_fee,
            operation_delayed_unstake_fee,
        )?;

        Ok(())
    }
}

fn set_fee_checked(
    current_value: &mut u8,
    new_value: Option<u8>,
) -> std::result::Result<(), ReferralError> {
    if let Some(new_fee) = new_value {
        // the fee is calculated as basis points
        if new_fee > MAX_OPERATION_FEE_POINTS {
            msg!(
                "Operation fee value {}bp is over maximal permitted {}bp",
                new_fee,
                MAX_OPERATION_FEE_POINTS
            );
            return Err(ReferralOperationFeeOverMax);
        }
        *current_value = new_fee;
    }
    Ok(())
}
