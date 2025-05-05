use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};
use anchor_spl::{token::{self, Approve, Mint, Token, TokenAccount, Transfer}, token_2022::Token2022};

use crate::states::{Config, Investment, InvestmentStatus, Payment};

const SWAP_EXACT_IN: [u8; 8] = [104, 104, 131, 86, 161, 189, 180, 216];

pub fn invest_gap(
    ctx: Context<InvestGap>,
    amount: u64,
) -> Result<()> {

    // 1. move funds: if usd*, move to vault, and other stable coins, swap.
    let src_mint = ctx.accounts.borrower_token.mint;
    let is_usd_star = src_mint == ctx.accounts.usd_star_mint.key();

    let binding = ctx.accounts.payment.key();
    let vault_seeds: &[&[u8]] = &[
        b"investment_vault",
        binding.as_ref(),
        &[ctx.bumps.investment_vault]
    ];

    // A. borrower -> programt temp account
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.borrower_token.to_account_info(),
                to: if is_usd_star {
                    ctx.accounts.investment_vault.to_account_info()
                } else {
                    ctx.accounts.pool_source.to_account_info()
                },
                authority: ctx.accounts.borrower.to_account_info(),
            }
        ),
        amount
    )?;

    // B. if usdc, usdt: swap -> USD*
    if !is_usd_star {
        token::approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Approve {
                    to: ctx.accounts.pool_source.to_account_info(),
                    delegate: ctx.accounts.borrower.to_account_info(),
                    authority: ctx.accounts.pool_source.to_account_info(),
                },
            ),
            amount,
        )?;

        // payload to SwapExactInHintlessData
        let swap_data = {
            #[derive(AnchorSerialize)]
            struct SwapPayload {
                in_index: u8,
                out_index: u8,
                exact_amount_in: u64,
                min_amount_out: u64
            }
            let p = SwapPayload {
                in_index: 0,
                out_index: 1,
                exact_amount_in: amount,
                min_amount_out: 0,
            };
            let mut v = SWAP_EXACT_IN.to_vec();
            v.extend(p.try_to_vec()?);
            v
        };

        let ix_accounts = vec![
            AccountMeta::new(ctx.accounts.num_pool.key(), true),
            AccountMeta::new(ctx.accounts.usdc_mint.key(), false),          // in_mint
            AccountMeta::new(ctx.accounts.usd_star_mint.key(), false),      // out_mint
            AccountMeta::new(ctx.accounts.pool_source.key(), false),        // in_trader
            AccountMeta::new(ctx.accounts.investment_vault.key(), false),     // out_trader
            AccountMeta::new_readonly(ctx.accounts.numeraire_config.key(), false),
            AccountMeta::new(ctx.accounts.borrower.key(), true),            // payer = borrower
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false),
        ];

        let ix = Instruction {
            program_id: *ctx.accounts.num_pool.owner,
            accounts: ix_accounts,
            data: swap_data,
        };

        invoke_signed(
            &ix,
            &[
                ctx.accounts.num_pool.to_account_info(),
                ctx.accounts.pool_source.to_account_info(),
                ctx.accounts.investment_vault.to_account_info(),
                ctx.accounts.numeraire_config.to_account_info(),
                ctx.accounts.borrower.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
            &[vault_seeds],
        )?;
    }
   
    // create investment record

    let investment = &mut ctx.accounts.investment;
    investment.borrower = ctx.accounts.borrower.key();
    investment.amount = amount;
    investment.start_ts = Clock::get()?.unix_timestamp;
    investment.status = InvestmentStatus::Locked;

    // link payment and borrower
    let payment = &mut ctx.accounts.payment;
    payment.borrower = ctx.accounts.borrower.key();

    Ok(())
}

#[derive(Accounts)]
pub struct InvestGap<'info> {
    #[account(mut)]
    pub payment: Account<'info, Payment>,

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

    // usd* vault
    #[account(
        mut,
        seeds = [b"investment_vault", payment.key().as_ref()],
        bump
    )]
    pub investment_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(mut)]
    pub borrower_token: Account<'info, TokenAccount>,


    // mints
    pub usdc_mint: Account<'info, Mint>,
    pub usdt_mint: Account<'info, Mint>,
    pub usd_star_mint: Account<'info, Mint>,

    // Numerarire CPI
    pub num_pool: AccountInfo<'info>,
    pub numeraire_config: AccountInfo<'info>,
    
    #[account(mut)]
    pub pool_source: Account<'info, TokenAccount>,

    #[account(mut)]
    pub pool_destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}