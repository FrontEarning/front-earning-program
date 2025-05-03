use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::states::{payment::*, investment::*};
use crate::error::CustomError;


#[derive(Accounts)]
pub struct Settle<'info> {
    #[account(mut)]
    pub payment: Account<'info, Payment>,
    #[account(
        mut,
        seeds=[b"vault", payment.key().as_ref()],
        bump
    )]
    pub vault_usd_star: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_receive: Account<'info, TokenAccount>,
    #[account(mut)]
    pub borrower_receive: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}