use anchor_lang::prelude::AccountInfo;
use anchor_lang::CpiContext;
use anchor_lang::{solana_program::entrypoint::ProgramResult, Accounts};

pub use spl_associated_token_account::{create_associated_token_account, ID};

pub fn create<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, Create<'info>>,
) -> ProgramResult {
    let ix = create_associated_token_account(
        ctx.accounts.payer.key,
        ctx.accounts.authority.key,
        ctx.accounts.mint.key,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.payer,
            ctx.accounts.associated_token,
            ctx.accounts.authority,
            ctx.accounts.mint,
            ctx.accounts.system_program,
            ctx.accounts.token_program,
            ctx.accounts.rent,
        ],
        ctx.signer_seeds,
    )
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Create<'info> {
    pub payer: AccountInfo<'info>,
    pub associated_token: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}
