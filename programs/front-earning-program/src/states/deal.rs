use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DealStatus {
    Initialized,
    Funded,
    Deposited,
    Redeemed,
    Cancelled,
}

#[account]
pub struct Deal {
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub investor: Pubkey,
    pub price: u64,
    pub discount_bps: u16,
    pub seller_share_bps: u16,
    pub deposited_amount: u64,
    pub maturity_ts: i64,
    pub status: DealStatus,
}

impl Deal {
    pub const LEN: usize = 8 + 32 * 3 + 8 + 2 + 2 + 8 + 8 + 1;
}