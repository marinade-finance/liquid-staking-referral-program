use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use solana_program::program_pack::IsInitialized;

use crate::constant::*;
use crate::error::ReferralError::ReferralOperationFeeOverMax;
use crate::error::*;
use crate::states::{GlobalState, ReferralState};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    #[account(zero)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    #[account()]
    pub msol_mint_account: CpiAccount<'info, Mint>,
}
impl<'info> Initialize<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.global_state.admin_account = self.admin_account.key();
        self.global_state.msol_mint_account = self.msol_mint_account.key();

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
        has_one = admin_account,
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account, signer
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    #[account(zero)] // must be created but empty, ready to be initialized
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // partner main account
    #[account()]
    pub partner_account: AccountInfo<'info>,

    // partner mSOL beneficiary token account
    #[account()]
    pub msol_token_partner_account: CpiAccount<'info, TokenAccount>,
}

impl<'info> InitReferralAccount<'info> {
    pub fn process(&mut self, partner_name: String) -> ProgramResult {
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

        self.referral_state.partner_account = self.partner_account.key();
        self.referral_state.msol_token_partner_account = self.msol_token_partner_account.key();

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

        self.referral_state.operation_deposit_sol_fee = DEFAULT_OPERATION_FEE_POINTS;
        self.referral_state.operation_deposit_stake_account_fee = DEFAULT_OPERATION_FEE_POINTS;
        self.referral_state.operation_liquid_unstake_fee = DEFAULT_OPERATION_FEE_POINTS;
        self.referral_state.operation_delayed_unstake_fee = DEFAULT_OPERATION_FEE_POINTS;

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
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // partner main account
    #[account()]
    pub new_partner_account: AccountInfo<'info>,

    // partner mSOL beneficiary token account
    #[account()]
    pub new_msol_token_partner_account: CpiAccount<'info, TokenAccount>,
}
impl<'info> UpdateReferral<'info> {
    pub fn process(
        &mut self,
        pause: bool,
        operation_deposit_sol_fee: Option<u8>,
        operation_deposit_stake_account_fee: Option<u8>,
        operation_liquid_unstake_fee: Option<u8>,
        operation_delayed_unstake_fee: Option<u8>,
    ) -> ProgramResult {
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
