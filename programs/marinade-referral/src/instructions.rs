use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{associated_token::Create as CreateAssociatedTokenAccount, states::*};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    // mSOL mint
    pub msol_mint: CpiAccount<'info, Mint>,

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
    pub msol_mint: CpiAccount<'info, Mint>,

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
        // has_one = partner_account @ ReferralError::AccessDenied,
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
        // has_one = partner_account @ ReferralError::AccessDenied,
    )]
    pub state: ProgramAccount<'info, ReferralState>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
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
    pub marinade_finance_state: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

    // Marinade main program ID
    pub marinade_finance_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DepositWrapper<'info> {
    pub state: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,
    pub reserve_pda: AccountInfo<'info>,
    pub transfer_from: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

impl<'info> Deposit<'info> {
    pub fn into_deposit_sol_cpi_ctx(&self) -> CpiContext<'_, '_, '_, 'info, DepositWrapper<'info>> {
        let cpi_accounts = DepositWrapper {
            state: self.marinade_finance_state.clone(),
            msol_mint: self.msol_mint.clone(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.clone(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.clone(),
            liq_pool_msol_leg_authority: self.liq_pool_msol_leg_authority.clone(),
            reserve_pda: self.reserve_pda.clone(),
            transfer_from: self.transfer_from.clone(),
            mint_to: self.mint_to.clone(),
            msol_mint_authority: self.msol_mint_authority.clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
        };

        CpiContext::new(self.marinade_finance_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct DepositStakeAccount<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
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
    pub marinade_finance_state: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,

    // Marinade main program ID
    pub marinade_finance_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct LiquidUnstake<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
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
    pub marinade_finance_state: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

    // Marinade main program ID
    pub marinade_finance_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct LiquidUnstakeWrapper<'info> {
    pub get_msol_from_authority: AccountInfo<'info>, //burn_msol_from owner or delegate_authority
    pub msol_mint: AccountInfo<'info>,
    pub get_msol_from: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub treasury_msol_account: AccountInfo<'info>,
    pub transfer_sol_to: AccountInfo<'info>,
    pub state: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

impl<'info> LiquidUnstake<'info> {
    pub fn into_liquid_unstake_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, LiquidUnstakeWrapper<'info>> {
        let cpi_accounts = LiquidUnstakeWrapper {
            state: self.marinade_finance_state.clone(),
            msol_mint: self.msol_mint.clone(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.clone(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.clone(),
            get_msol_from: self.get_msol_from.clone(),
            get_msol_from_authority: self.get_msol_from_authority.clone(),
            transfer_sol_to: self.transfer_sol_to.clone(),
            treasury_msol_account: self.treasury_msol_account.clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
        };

        CpiContext::new(self.marinade_finance_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct RequestTransfer<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
    )]
    pub state: ProgramAccount<'info, ReferralState>,

    #[account(mut, signer)]
    pub request_account: AccountInfo<'info>,

    pub treasury_msol_account: AccountInfo<'info>,
}

//-----------------------------------------------------
