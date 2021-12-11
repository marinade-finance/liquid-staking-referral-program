///seeds
pub const GLOBAL_STATE_SEED: &[u8] = b"mrp_initialize";
pub const REFERRAL_STATE_SEED: &[u8] = b"mrp_create_referral";

///mSOL Mint address
pub const MSOL_MINT_ADDRESS: &str = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So";

///30 days in seconds, 3600 * 24 * 30
pub const DEFAULT_TRANSFER_DURATION: u32 = 2_592_000;

///Base % cut for the partner
pub const DEFAULT_BASE_FEE_POINTS: u32 = 1_000;

///Max % cut for the partner
pub const DEFAULT_MAX_FEE_POINTS: u32 = 10_000;

///Net stake target for max %
pub const DEFAULT_MAX_NET_STAKE: u64 = 1_000_000_000_000_000;
