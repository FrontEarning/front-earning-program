use anchor_lang::prelude::*;

#[account]
pub struct LiquidityPool {
    pub total_liquidity: u64,   // total USD* inside the pool vault
    pub total_shares: u64,       // share accounting (1e6 precision)
    pub allocated: u64,   // USD* currently matched to gap loans
    pub bump: u8,               // PDA bump
}

impl LiquidityPool {
    pub const SEED: &'static [u8] = b"pool";
    pub const LEN: usize = 8 + 8 + 8 + 8 + 1;
}