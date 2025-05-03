use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DiscountEntry {
    pub mint: Pubkey,
    pub discount_bps: u16,
}

#[account]
pub struct GlobalConfig {
    pub owner: Pubkey,
    pub discounts: Vec<DiscountEntry>,
    pub maturity_period_secs: i64, // default 31_536_000 (1yr = 365d)
}

impl GlobalConfig {
    pub const LEN_FIXED: usize = 8 + 32 + 8;
}