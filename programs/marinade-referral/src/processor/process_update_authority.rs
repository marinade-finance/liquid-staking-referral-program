use anchor_lang::prelude::*;
use anchor_spl::associated_token;

use crate::instructions::*;

pub fn process_update_authority(ctx: Context<UpdateAuthority>) -> ProgramResult {
    ctx.accounts.state.partner_account = *ctx.accounts.new_partner_account.key;
    ctx.accounts.state.beneficiary_account = *ctx.accounts.new_beneficiary_account.key;

    // create associated token account for partner
    if **ctx.accounts.new_beneficiary_account.lamports.borrow() == 0_u64 {
        associated_token::create(ctx.accounts.into_create_associated_token_account_ctx())?;
    }

    Ok(())
}
