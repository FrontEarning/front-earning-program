use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum InvestmentStatus {
    Active,
    Withdrawn,
}

#[account]
pub struct Investment {
    pub borrower: Pubkey,
    pub amount_usd_star: u64,
    pub start_ts: i64,
    pub unlock_ts: i64,
    pub status: InvestmentStatus,
}

impl Investment {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 1;
}

