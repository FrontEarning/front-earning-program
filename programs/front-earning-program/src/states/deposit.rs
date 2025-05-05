use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DepositStatus {
    Active,
    Withdrawn,
}

#[account]
pub struct BorrowerDeposit {
    pub owner: Pubkey,
    pub shares: u64,
    pub start_ts: i64,
    pub status: DepositStatus,
    pub bump: u8,
}

impl BorrowerDeposit {
    pub const SEED: &'static [u8] = b"deposit";
    pub const LEN: usize = 32 + 8 + 8 + 1;
}