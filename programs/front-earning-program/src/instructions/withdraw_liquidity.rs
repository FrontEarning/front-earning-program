use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{states::{deposit::BorrowerDeposit, pool::*, DepositStatus}, CustomError};

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut, seeds=[b"pool"], bump = pool.bump)]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        seeds=[b"deposit", owner.key().as_ref()], 
        bump = deposit.bump)
    ]
    pub deposit: Account<'info, BorrowerDeposit>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub vault_usd_star: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>) -> Result<()> {
    let dep_shares = ctx.accounts.deposit.shares;
    require!(dep_shares > 0, CustomError::NothingToWithdraw);

    let pool = &mut ctx.accounts.pool;
    // proportionate amount
    let amount = dep_shares.checked_mul(pool.total_liquidity).unwrap() / pool.total_shares;

    pool.total_liquidity -= amount;
    pool.total_shares -= dep_shares;

    ctx.accounts.deposit.status = DepositStatus::Withdrawn;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_usd_star.to_account_info(),
                to: ctx.accounts.owner_token.to_account_info(),
                authority: ctx.accounts.vault_usd_star.to_account_info(), // PDA self-owned
            }
        ),
        amount,
    )?;

    Ok(())
}