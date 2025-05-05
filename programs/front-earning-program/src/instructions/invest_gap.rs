use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{states::{Config, Investment, InvestmentStatus}, CustomError};

pub fn invest_gap(
    ctx: Context<InvestGap>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, CustomError::InputTooSmall);
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.borrower_token.to_account_info(),
                to: ctx.accounts.investment_vault.to_account_info(),
                authority: ctx.accounts.borrower.to_account_info(),
            }
        ),
        amount,
    )?;

    let investment = &mut ctx.accounts.investment;
    investment.borrower = ctx.accounts.borrower.key();
    investment.amount = amount;
    investment.start_ts = Clock::get()?.unix_timestamp;
    investment.status = InvestmentStatus::Locked;

    Ok(())
}

#[derive(Accounts)]
pub struct InvestGap<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = borrower,
        space = 8 + Investment::LEN,
        seeds = [b"investment", borrower.key().as_ref()],
        bump
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        mut,
        seeds = [b"investment_vault", borrower.key().as_ref()],
        bump
    )]
    pub investment_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(mut)]
    pub borrower_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}