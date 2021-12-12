use anchor_lang::{
    prelude::*,
    InstructionData,
    solana_program::instruction::{Instruction},
};

// helper function to use anchor_lang::solana_program::program::invoke_signed
// with anchor_lang::CpiContext
pub fn invoke_signed<'info, T: Accounts<'info> + ToAccountInfos<'info> + ToAccountMetas, AnchorInstruction:InstructionData>(
    cpi_ctx: CpiContext<'_, '_, '_, 'info, T>,
    instruction_data: AnchorInstruction,
) -> ProgramResult {
    let ix = Instruction {
        program_id: *cpi_ctx.program.key,
        accounts: cpi_ctx.accounts.to_account_metas(None),
        data: instruction_data.data(),
    };
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &cpi_ctx.to_account_infos(),
        cpi_ctx.signer_seeds,
    )
}
