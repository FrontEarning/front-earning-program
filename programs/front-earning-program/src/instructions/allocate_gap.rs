use anchor_lang::prelude::*;
use crate::{states::{pool::LiquidityPool, Payment}, CustomError};

pub fn handle_allocate_gap(
    payment: &mut Payment,
    pool: &mut LiquidityPool,
    gap_amount: u64,
) -> Result<()> {
    require!(pool.total_liquidity - pool.allocated >= gap_amount, CustomError::InsufficientLiquidity);
    // shares cost
    let share_cost = gap_amount.checked_mul(pool.total_shares).unwrap() / pool.total_liquidity;

    pool.total_liquidity -= gap_amount;
    pool.total_shares -= share_cost;
    pool.allocated += gap_amount;

    payment.gap_amount = gap_amount;
    payment.gap_shares = share_cost;

    Ok(())
}

pub fn allocate_gap(
    ctx: Context<AllocateGap>,
    gap_amount: u64,
) -> Result<()> { // returns consumed_shares
    let pool = &mut ctx.accounts.pool;
    require!(pool.total_liquidity - pool.allocated >= gap_amount, CustomError::InsufficientLiquidity);

    let payment = &mut ctx.accounts.payment;
    handle_allocate_gap(payment, pool, gap_amount)
}

#[derive(Accounts)]
pub struct AllocateGap<'info> {
    #[account(mut)]
    pub payment: Account<'info, Payment>,
    
    #[account(mut, seeds=[b"pool"], bump = pool.bump)]
    pub pool: Account<'info, LiquidityPool>,
}
