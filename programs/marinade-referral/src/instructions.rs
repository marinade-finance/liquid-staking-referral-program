#![allow(unused_imports)]

use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::Create as CreateAssociatedTokenAccount;
use anchor_spl::token::{self, Mint, TokenAccount};

use crate::{constant::*, error::*, processor::*, states::*};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    // mSOL mint
    pub msol_mint: Account<'info, Mint>,

    // beneficiary ATA
    #[account(mut)]
    pub beneficiary_account: AccountInfo<'info>,

    // partner account, signer
    #[account(mut, signer)]
    pub partner_account: AccountInfo<'info>,

    // referral state
    #[account(zero)]
    pub state: ProgramAccount<'info, ReferralState>,

    pub system_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

impl<'info> Initialize<'info> {
    pub fn into_create_associated_token_account_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, CreateAssociatedTokenAccount<'info>> {
        let cpi_accounts = CreateAssociatedTokenAccount {
            payer: self.partner_account.clone(),
            associated_token: self.beneficiary_account.clone(),
            authority: self.partner_account.clone(),
            mint: self.msol_mint.to_account_info().clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
            rent: self.rent.clone(),
        };

        CpiContext::new(self.associated_token_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    // mSOL mint
    pub msol_mint: Account<'info, Mint>,

    // beneficiary ATA
    #[account(mut)]
    pub new_beneficiary_account: AccountInfo<'info>,

    // new authority
    pub new_partner_account: AccountInfo<'info>,

    // partner account, signer
    #[account(mut, signer)]
    pub partner_account: AccountInfo<'info>,

    // referral state
    #[account(
        mut,
        has_one = partner_account @ CommonError::UnexpectedAccount,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    pub system_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

impl<'info> UpdateAuthority<'info> {
    pub fn into_create_associated_token_account_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, CreateAssociatedTokenAccount<'info>> {
        let cpi_accounts = CreateAssociatedTokenAccount {
            payer: self.partner_account.clone(),
            associated_token: self.new_beneficiary_account.clone(),
            authority: self.new_partner_account.clone(),
            mint: self.msol_mint.to_account_info().clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
            rent: self.rent.clone(),
        };

        CpiContext::new(self.associated_token_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Update<'info> {
    // partner account, signer
    #[account(mut, signer)]
    pub partner_account: AccountInfo<'info>,

    // referral state
    #[account(
        mut,
        has_one = partner_account @ CommonError::UnexpectedAccount,
    )]
    pub state: ProgramAccount<'info, ReferralState>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        constraint = !state.pause,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(signer)]
    pub transfer_from: AccountInfo<'info>,

    pub msol_mint: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,
    pub reserve_pda: AccountInfo<'info>,
    pub marinade_state: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct DepositStakeAccount<'info> {
    #[account(
        mut,
        constraint = !state.pause,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(signer)]
    pub stake_authority: AccountInfo<'info>,

    #[account(signer)]
    pub rent_payer: AccountInfo<'info>,

    pub validator_list: AccountInfo<'info>,
    pub stake_list: AccountInfo<'info>,
    pub stake_account: AccountInfo<'info>,
    pub duplication_flag: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub marinade_state: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct LiquidUnstake<'info> {
    #[account(
        mut,
        constraint = !state.pause,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(signer)]
    pub get_msol_from_authority: AccountInfo<'info>, //burn_msol_from owner or delegate_authority
    pub msol_mint: AccountInfo<'info>,
    pub get_msol_from: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub treasury_msol_account: AccountInfo<'info>,
    pub transfer_sol_to: AccountInfo<'info>,
    pub marinade_state: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

//-----------------------------------------------------
