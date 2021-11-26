#![allow(unused_imports)]

use anchor_lang::prelude::*;

use crate::{constant::*, error::*, fees::Fee, instructions::*, states::*};

pub fn process_update_partner(ctx: Context<UpdatePartner>) -> ProgramResult {
    ctx.accounts.state.partner_account = *ctx.accounts.new_partner_account.key;

    Ok(())
}
