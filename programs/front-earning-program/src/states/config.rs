use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub owner: Pubkey,
    pub usdc_discount_bps: u16,
    pub usdt_discount_bps: u16,
    pub usd_star_discount_bps: u16,
    pub invest_lock_secs: i64,  // default = 1yr, mvp = ?
    pub settle_wait_secs: i64,  // default = 1hr, mvp = ?
}

impl Config {
    pub const LEN: usize = 8 + 32 + 2*3 + 8 + 8;
}