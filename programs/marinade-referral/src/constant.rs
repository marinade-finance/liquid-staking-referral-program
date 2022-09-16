use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

///Base % cut for the partner
pub const DEFAULT_BASE_FEE_POINTS: u32 = 1_000; // 10%

///Max % cut for the partner
pub const DEFAULT_MAX_FEE_POINTS: u32 = 10_000; // 100%

pub const DEFAULT_OPERATION_FEE_POINTS: u8 = 0; // 0%
pub const MAX_OPERATION_FEE_POINTS: u8 = 50; // 0.5%

///Net stake target for max %
pub const DEFAULT_MAX_NET_STAKE: u64 = 1_000_000 * LAMPORTS_PER_SOL;
