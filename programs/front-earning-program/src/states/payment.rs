use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Initialized,
    Funded,
    Settled,
}

#[account]
pub struct Payment {
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub borrower: Pubkey, // gap-funder
    pub amount: u64,
    pub owner: u64,
    pub paid_amount: u64,
    pub paid_mint: Pubkey,
    pub discount_bps: u16,
    pub start_ts: i64,
    pub status: PaymentStatus,
    pub gap_amount: u64,
    pub gap_shares: u64,
}

impl Payment { 
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 * 5 + 2 + 1 + 1;
}
