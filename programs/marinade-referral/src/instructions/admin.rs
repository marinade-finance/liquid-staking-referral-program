use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use solana_program::program_pack::IsInitialized;
use solana_program::system_program;

use crate::constant::*;
use crate::error::ReferralError::{
    ExceededNumberForemenSignerKeys, NeitherAdminNorForemanReferralState,
    ReferralOperationFeeOverMax,
};
use crate::error::*;
use crate::states::{GlobalState, ReferralState, MAX_FOREMEN_NUMBER};

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
    // list of foremen accounts are expected to be provided as remaining_accounts of the context
}
impl<'info> Initialize<'info> {
    pub fn process(&mut self, foremen_pubkeys: Vec<Pubkey>) -> ProgramResult {
        self.global_state.admin_account = self.admin_account.key();
        self.global_state.msol_mint_account = self.msol_mint_account.key();

        // verify if the account that should be considered as MSOL mint is an active mint account
        if !self.msol_mint_account.is_initialized() {
            return Err(ReferralError::NotInitializedMintAccount.into());
        }

        // foremen accounts that are capable to change referral account config
        set_foremen(foremen_pubkeys, &mut self.global_state)?;

        Ok(())
    }
}

fn set_foremen<'info>(
    foremen_pubkeys: Vec<Pubkey>,
    global_state: &mut ProgramAccount<'info, GlobalState>,
) -> std::result::Result<(), ReferralError> {
    if foremen_pubkeys.len() > MAX_FOREMEN_NUMBER {
        return Err(ExceededNumberForemenSignerKeys);
    }
    for i in 0..MAX_FOREMEN_NUMBER - 1 {
        if let Some(foreman_pubkey) = foremen_pubkeys.get(i) {
            global_state.foremen[i] = *foreman_pubkey;
        } else {
            global_state.foremen[i] = system_program::ID;
        }
    }
    Ok(())
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct InitReferralAccount<'info> {
    // global state
    #[account()]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // an admin or a foreman account; account that is permitted to init the referral state
    #[account(signer)]
    pub admin_or_foreman_account: AccountInfo<'info>,

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

        // verify whether the signer is permitted to init the referral account
        check_admin_foreman_permission(&self.admin_or_foreman_account, &self.global_state)?;

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

        Ok(())
    }
}

fn check_admin_foreman_permission<'info>(
    account_to_verify: &AccountInfo<'info>,
    global_state: &ProgramAccount<'info, GlobalState>,
) -> std::result::Result<(), ReferralError> {
    let mut permissioned_accounts: Vec<Pubkey> = vec![];
    permissioned_accounts.push(global_state.admin_account);
    for i in 0..MAX_FOREMEN_NUMBER - 1 {
        if global_state.foremen.len() > i && global_state.foremen[i] != Pubkey::default() {
            permissioned_accounts.push(global_state.foremen[i]);
        }
    }
    for permissioned_account in permissioned_accounts {
        if permissioned_account.key() == *account_to_verify.key && account_to_verify.is_signer {
            return Ok(());
        }
    }
    return Err(NeitherAdminNorForemanReferralState);
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

//--------------------------------------
#[derive(Accounts)]
pub struct ChangeForemen<'info> {
    // global state
    #[account()]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // an admin or a foreman account; accounts with permission to changes
    #[account(signer)]
    pub admin_or_foreman_account: AccountInfo<'info>,
    // the list of new foremen accounts to be configured
    // are expected to be provided as remaining_accounts of the context
}
impl<'info> ChangeForemen<'info> {
    pub fn process(&mut self, foremen_pubkeys: Vec<Pubkey>) -> ProgramResult {
        check_admin_foreman_permission(&self.admin_or_foreman_account, &self.global_state)?;
        set_foremen(foremen_pubkeys, &mut self.global_state)?;
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateReferral<'info> {
    // global state
    #[account()]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account
    #[account(signer)]
    pub admin_or_foreman_account: AccountInfo<'info>,

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
        check_admin_foreman_permission(&self.admin_or_foreman_account, &self.global_state)?;

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
