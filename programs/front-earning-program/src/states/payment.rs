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
}

impl Payment { 
    pub const LEN: usize = 8 + 32*3 + 8 + 8 + 32 + 2 + 8 + 1; 
}
