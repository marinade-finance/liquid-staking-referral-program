#![allow(unused_imports)]

use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount};

use crate::{constant::*, error::*, processor::*, states::*};

//-----------------------------------------------------
#[derive(Accounts)]
#[instruction(
    ref_code: String,
    referral_bump: u8,
    beneficiary_bump: u8,
)]
pub struct Initialize<'info> {
    // referral state
    #[account(
        init,
        payer = partner_account,
        space = 8 + size_of::<ReferralState>(),
        seeds = [
            b"referral".as_ref(),
            ref_code.as_bytes().as_ref(),
        ],
        bump = referral_bump,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    // mSOL mint
    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,
    // mSOL vault account
    #[account(
        init,
        payer = partner_account,
        seeds = [
            b"beneficiary".as_ref(),
            ref_code.as_bytes().as_ref(),
        ],
        bump = beneficiary_bump,
        token::mint = msol_mint,
        token::authority = beneficiary,
    )]
    pub beneficiary: Account<'info, TokenAccount>,

    // partner account, signer
    #[account(mut, signer)]
    pub partner_account: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        constraint = state.partner_account == *partner_account.key @ CommonError::UnexpectedAccount,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    // partner account, signer
    #[account(mut, signer)]
    pub partner_account: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,

    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    #[account(mut, signer)]
    pub transfer_from: AccountInfo<'info>,

    #[account(mut)]
    pub mint_to: Account<'info, TokenAccount>,

    pub msol_mint_authority: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct DepositStakeAccount<'info> {
    #[account(mut)]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,

    #[account(mut)]
    pub stake_account: Account<'info, StakeWrapper>,
    #[account(signer)]
    pub stake_authority: AccountInfo<'info>,
    #[account(mut)]
    pub duplication_flag: AccountInfo<'info>,
    #[account(mut, signer)]
    pub rent_payer: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub mint_to: Account<'info, TokenAccount>,

    pub msol_mint_authority: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct LiquidUnstake<'info> {
    #[account(mut)]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    #[account(mut)]
    pub get_msol_from: Account<'info, TokenAccount>,
    #[account(signer)]
    pub get_msol_from_authority: AccountInfo<'info>, //burn_msol_from owner or delegate_authority

    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

//-----------------------------------------------------
