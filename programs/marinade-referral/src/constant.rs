use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

///Global state ID
pub const GLOBAL_STATE_ADDRESS: &str = "mRg6bDsAd5uwERAdNTynoUeRbqQsLa7yzuK2kkCUPGW";

///mSOL treasury PDA
pub const MSOL_TREASURY_AUTH_SEED: &[u8] = b"mr_treasury";

///30 days in seconds, 3600 * 24 * 30
pub const DEFAULT_TRANSFER_DURATION: u32 = 2_592_000;

///Base % cut for the partner
pub const DEFAULT_BASE_FEE_POINTS: u32 = 1_000; // 10%

///Max % cut for the partner
pub const DEFAULT_MAX_FEE_POINTS: u32 = 10_000; // 100%

///Net stake target for max %
pub const DEFAULT_MAX_NET_STAKE: u64 = 1_000_000 * LAMPORTS_PER_SOL;
