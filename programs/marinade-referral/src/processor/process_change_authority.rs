use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token;

use crate::{associated_token, constant::*, error::*, instructions::*};

///change admin and its beneficiary ATA
pub fn process_change_authority(ctx: Context<ChangeAuthority>) -> ProgramResult {
    // check if the mSOL mint is actually the mSOL mint
    // if Pubkey::from_str(MSOL_MINT_ADDRESS).unwrap() != ctx.accounts.msol_mint.key() {
    //     return Err(ReferralError::AccessDenied.into());
    // }

    // verify msol_mint_authority
    if ctx.accounts.msol_mint.mint_authority.unwrap()
        != Pubkey::from_str(MSOL_MINT_AUTHORITY_ADDRESS).unwrap()
    {
        return Err(ReferralError::AccessDenied.into());
    }

    // check associated_token_program to see if they're correct
    if ctx.accounts.associated_token_program.key() != associated_token::ID {
        return Err(ReferralError::AccessDenied.into());
    }

    // check token_program to see if they're correct
    if ctx.accounts.token_program.key() != token::ID {
        return Err(ReferralError::AccessDenied.into());
    }

    // check authority
    if ctx
        .accounts
        .state
        .partner_account
        .ne(ctx.accounts.partner_account.key)
    {
        return Err(ReferralError::AccessDenied.into());
    }

    // create associated token account for partner
    if **ctx.accounts.new_beneficiary_account.lamports.borrow() == 0_u64 {
        associated_token::create(ctx.accounts.into_create_associated_token_account_ctx())?;
    }

    ctx.accounts.state.partner_account = *ctx.accounts.new_partner_account.key;
    ctx.accounts.state.beneficiary_account = *ctx.accounts.new_beneficiary_account.key;

    Ok(())
}
