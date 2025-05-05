use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Approve};
use crate::{allocate_gap, handle_allocate_gap};
use crate::states::{payment::*, Config, LiquidityPool};
use crate::error::CustomError;

const SWAP_EXACT_IN_DISCRIMINATOR: [u8; 8] = [104, 104, 131, 86, 161, 189, 180, 216];

pub fn execute_payment(
    ctx: Context<ExecutePayment>,
    amount: u64,
    in_index: u8,
    out_index: u8,
    min_out: u64,
) -> Result<()> {
    // 1. Discount & gap
    let config = &ctx.accounts.config;
    let discount_bps = config.usdc_discount_bps as u64; // TODO : match by other token mints
    let discounted = amount * (10_000 - discount_bps) / 10_000;
    let gap_amount = amount.checked_sub(discounted).unwrap();

    require!(ctx.accounts.payment.status == PaymentStatus::Initialized, CustomError::InvalidState);

    // 2. Token flow: Buyer -> vault_in (buyer token), token amount = discounted
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_token.to_account_info(),
                to: ctx.accounts.vault_in.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        discounted,
    )?;

    // 3. approve vault for pool pull (for swap)
    let binding = ctx.accounts.payment.key();
    let vault_seeds = &[
        b"vault",
        binding.as_ref(),
        &[ctx.bumps.vault_in],
    ];
    token::approve(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Approve {
                to: ctx.accounts.vault_in.to_account_info(),
                delegate: ctx.accounts.payer.to_account_info(),
                authority: ctx.accounts.vault_in.to_account_info(),
            },
            &[vault_seeds],
        ),
        discounted,
    )?;

    // 4. call swap_exact_in via numeraire(perena) CPI
    let mut data = SWAP_EXACT_IN_DISCRIMINATOR.to_vec();
    data.extend(&SwapExactInHintlessData {
        in_index,
        out_index,
        exact_amount_in: discounted,
        min_amount_out: min_out,
    }.try_to_vec()?);

    let ix = Instruction {
        program_id: *ctx.accounts.pool.to_account_info().owner,
        accounts: vec![
            AccountMeta::new(ctx.accounts.pool.key(), true),
            AccountMeta::new(ctx.accounts.in_mint.key(), false),
            AccountMeta::new(ctx.accounts.out_mint.key(), false),
            AccountMeta::new(ctx.accounts.vault_in.key(), false),
            AccountMeta::new(ctx.accounts.vault_out.key(), false),
            AccountMeta::new_readonly(ctx.accounts.numeraire_config.key(), false),
            AccountMeta::new(ctx.accounts.vault_in.key(), true), // payer
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false),
        ],
        data,
    };
    invoke_signed(&ix, &[
        ctx.accounts.pool.to_account_info().clone(),
        ctx.accounts.in_mint.clone(),
        ctx.accounts.out_mint.clone(),
        ctx.accounts.vault_in.to_account_info(),
        ctx.accounts.vault_out.to_account_info(),
        ctx.accounts.numeraire_config.clone(),
        ctx.accounts.vault_in.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_2022_program.clone(),
    ], &[vault_seeds])?;

    // 5. Draw gap from liquidity pool (USD*) -> vault_out
    if gap_amount > 0 {
        // a) Update pool + payment shares via CPI to internal module
        handle_allocate_gap(&mut ctx.accounts.payment, &mut ctx.accounts.pool, gap_amount)?;

        // b) transfer USD* tokens from pool vault to vault_out
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_pool_usd_star.to_account_info(),
                    to: ctx.accounts.vault_out.to_account_info(),
                    authority: ctx.accounts.vault_pool_usd_star.to_account_info(),
                },
            ),
            gap_amount,
        )?;
    }
    
    // 6. record payment state
    let payment = &mut ctx.accounts.payment;
    payment.buyer = ctx.accounts.buyer.key();
    payment.paid_mint = ctx.accounts.in_mint.key();
    payment.paid_amount = discounted;
    payment.discount_bps = discount_bps as u16;
    payment.start_ts = Clock::get()?.unix_timestamp;
    payment.status = PaymentStatus::Funded;
    
    Ok(())
}

#[derive(Accounts)]
pub struct ExecutePayment<'info> {
    #[account(mut)]
    pub payment: Account<'info, Payment>,

    #[account(seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,

    #[account(mut, seeds=[b"pool"], bump = pool.bump)]
    pub pool: Account<'info, LiquidityPool>,

    #[account(mut, seeds=[b"vault_pool"], bump)]
    pub vault_pool_usd_star: Account<'info, TokenAccount>,

    // Token vault that temporarily hodls buyer token(PDA)
    #[account(
        mut,
        seeds=[b"vault", payment.key().as_ref()],
        bump
    )]
    pub vault_in: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds=[b"vault_out", payment.key().as_ref()],
        bump
    )]
    pub vault_out: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub buyer_token: Account<'info, TokenAccount>, // ATA for chosen token_mint

    // ---------- Numeraire pool & related (perena swap) ----------
    /// CHECK: pool
    #[account(mut)]
    pub numeraire_pool_account: AccountInfo<'info>,
    /// CHECK: in_mint (same as pay_mint)
    #[account(mut)]
    pub in_mint: AccountInfo<'info>,
    /// CHECK: out_mint (USD*)
    #[account(mut)]
    pub out_mint: AccountInfo<'info>,

    /// CHECK: numeraire config
    #[account(mut)]
    pub numeraire_config: AccountInfo<'info>,

    /// CHECK: payer (== vault_in)
    pub payer: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct SwapExactInHintlessData {
    in_index: u8,
    out_index: u8,
    exact_amount_in: u64,
    min_amount_out: u64,
}