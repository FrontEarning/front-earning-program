use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Initialized,
    Funded,
    Deposited,
    Settled,
}

#[account]
pub struct Payment {
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub borrower: Pubkey, // gap-funder
    pub price_usd: u64,
    pub discount_bps: u16,
    pub paid_mint: Pubkey,
    pub paid_amount: u64,
    pub usd_star_deposit: u64, // amount of USD* sent to vault
    pub created_ts: i64,
    pub maturity_ts: i64,
    pub status: PaymentStatus,
}

impl Payment {
    pub const LEN: usize = 8 + 32*3 + 8 + 2 + 32 + 8 + 8 + 8 + 8 + 1;
}
