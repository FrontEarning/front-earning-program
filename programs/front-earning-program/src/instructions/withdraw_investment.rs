use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{states::{Config, Investment, InvestmentStatus}, CustomError};

pub fn withdraw_investment(
    ctx: Context<WithdrawInvestment>,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let investment = &mut ctx.accounts.investment;
    require!(investment.status == InvestmentStatus::Locked, CustomError::InvalidState);
    let now = Clock::get()?.unix_timestamp;
    require!(now >= investment.start_ts + config.invest_lock_secs, CustomError::NotMature);

    let binding = ctx.accounts.borrower.key();
    let seeds = &[b"investment_vault", binding.as_ref(), &[ctx.bumps.investment_vault]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.investment_vault.to_account_info(),
            to: ctx.accounts.borrower_token.to_account_info(),
            authority: ctx.accounts.investment_vault.to_account_info(),
            },
            &[seeds],
        ),
        investment.amount,
    )?;

    investment.status = InvestmentStatus::Withdrawn;
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawInvestment<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        has_one = borrower
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        mut,
        seeds = [b"investment_vault", borrower.key().as_ref()],
        bump
    )]
    pub investment_vault: Account<'info, TokenAccount>,


    #[account(mut)]
    pub borrower_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub borrower: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}