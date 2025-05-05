use anchor_lang::prelude::*;
use anchor_spl::token::{self as token, Token, TokenAccount, Transfer};
use crate::{states::{payment::*, config::*}, error::CustomError};

pub fn settle(
    ctx: Context<Settle>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let config = &ctx.accounts.config;
    require!(ctx.accounts.payment.status == PaymentStatus::Funded, CustomError::InvalidState);
    require!(now > ctx.accounts.payment.start_ts + config.settle_wait_secs, CustomError::NotMature);
    
    let total = ctx.accounts.payment.paid_amount * 104 / 100;
    let interest = total - ctx.accounts.payment.paid_amount;
    let seller_share = interest / 2 + ctx.accounts.payment.paid_amount;
    let borrower_share = interest / 2;

    let binding = ctx.accounts.payment.key();
    let seeds = &[
        b"vault_out",
        binding.as_ref(),
        &[ctx.bumps.vault_usd_star]
    ];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_usd_star.to_account_info(),
                to: ctx.accounts.seller_token.to_account_info(),
                authority: ctx.accounts.vault_usd_star.to_account_info(),
            },
            &[seeds],
        ),
        seller_share
    )?;
    
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_usd_star.to_account_info(),
                to: ctx.accounts.borrower_token.to_account_info(),
                authority: ctx.accounts.vault_usd_star.to_account_info(),
            },
            &[seeds],
        ),
        borrower_share
    )?;

    let payment = &mut ctx.accounts.payment;
    payment.status = PaymentStatus::Settled;

    Ok(())
}


#[derive(Accounts)]
pub struct Settle<'info> {
    #[account(mut)]
    pub payment: Account<'info, Payment>,
    #[account(
        seeds=[b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds=[b"vault", payment.key().as_ref()],
        bump
    )]
    pub vault_usd_star: Account<'info, TokenAccount>,
    
    // payouts
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub seller_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub borrower_token: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}