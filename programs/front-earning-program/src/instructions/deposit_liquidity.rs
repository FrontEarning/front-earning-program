use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token::{self, Approve, Mint, Token, TokenAccount, Transfer};
use crate::states::DepositStatus;
use crate::states::{pool::*, deposit::BorrowerDeposit};
use crate::error::CustomError;

const SWAP_EXACT_IN: [u8;8] = [104,104,131,86,161,189,180,216];

pub fn deposit_liquidity(
    ctx: Context<DepositLiquidity>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, CustomError::InputTooSmall);
    let src_mint = ctx.accounts.borrower_source.mint;
    let is_usd_star = src_mint == ctx.accounts.usd_star_mint.key();

    // 1. move tokens
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.borrower_source.to_account_info(),
                to: if is_usd_star {
                    ctx.accounts.vault_usd_star.to_account_info()
                } else {
                    ctx.accounts.pool_source.to_account_info()
                },
                authority: ctx.accounts.borrower.to_account_info(),
            }
        ),
        amount,
    )?;

    // 2. swap if needed
    if !is_usd_star {
        token::approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Approve {
                    to: ctx.accounts.pool_source.to_account_info(),
                    delegate: ctx.accounts.borrower.to_account_info(),
                    authority: ctx.accounts.borrower.to_account_info(),
                },
            ),
            amount,
        )?;

        #[derive(AnchorSerialize)]
        struct SwapPayload { in_index: u8, out_index: u8, exact_amount_in: u64, min_amount_out: u64 }
        let mut data = SWAP_EXACT_IN.to_vec();
        data.extend(SwapPayload{in_index:0,out_index:1,exact_amount_in:amount,min_amount_out:0}.try_to_vec()?);

        let ix = Instruction {
            program_id: *ctx.accounts.num_pool.owner,
            accounts: vec![
                AccountMeta::new(ctx.accounts.num_pool.key(), true),
                AccountMeta::new(src_mint, false),
                AccountMeta::new(ctx.accounts.usd_star_mint.key(), false),
                AccountMeta::new(ctx.accounts.pool_source.key(), false),
                AccountMeta::new(ctx.accounts.vault_usd_star.key(), false),
                AccountMeta::new_readonly(ctx.accounts.numeraire_config.key(), false),
                AccountMeta::new(ctx.accounts.borrower.key(), true),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false),
            ],
            data,
        };
        invoke_signed(
            &ix,
            &[
                ctx.accounts.num_pool.to_account_info(),
                ctx.accounts.pool_source.to_account_info(),
                ctx.accounts.vault_usd_star.to_account_info(),
                ctx.accounts.numeraire_config.to_account_info(),
                ctx.accounts.borrower.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
            &[],
        )?;
    }

    // 3. mint share
    let pool = &mut ctx.accounts.pool;
    let shares = if pool.total_shares == 0 {
        amount // 1:1 first deposit
    } else {
        amount.checked_mul(pool.total_shares).unwrap() / pool.total_liquidity
    };

    pool.total_liquidity = pool.total_liquidity.checked_add(amount).unwrap();
    pool.total_shares = pool.total_shares.checked_add(shares).unwrap();

    let dep = &mut ctx.accounts.deposit;
    if dep.status == DepositStatus::Withdrawn {
        dep.shares = 0;
    }
    dep.owner = ctx.accounts.borrower.key();
    dep.shares = dep.shares.checked_add(shares).unwrap();
    dep.start_ts = Clock::get()?.unix_timestamp;
    dep.status = DepositStatus::Active;
    dep.bump = ctx.bumps.deposit.clone();
    Ok(())
}

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(mut)]
    pub borrower_source: Account<'info, TokenAccount>, // USDC/USDT/USD*

    #[account(
        mut,
        seeds=[LiquidityPool::SEED],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        init_if_needed,
        payer = borrower,
        space = 8 + BorrowerDeposit::LEN,
        seeds = [BorrowerDeposit::SEED, borrower.key().as_ref()],
        bump
    )]
    pub deposit: Account<'info, BorrowerDeposit>,

    #[account(
        mut,
        seeds=[b"pool_vault"], // program-owned USD* vault (one per pool)
        bump
    )]
    pub vault_usd_star: Account<'info, TokenAccount>,
    
    // mints
    pub usdc_mint: Account<'info, Mint>,
    pub usdt_mint: Account<'info, Mint>,
    pub usd_star_mint: Account<'info, Mint>,

    // Numeraire CPI
    /// CHECK: pool
    pub num_pool: AccountInfo<'info>,
    /// CHECK: config PDA
    pub numeraire_config: AccountInfo<'info>,
    #[account(mut)]
    pub pool_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    /// CHECK: token 2022 program
    pub token_2022_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}