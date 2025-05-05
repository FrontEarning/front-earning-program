use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, Approve};
use crate::states::{config::*, payment::*};
use crate::error::CustomError;

const SWAP_EXACT_IN_DISCRIMINATOR: [u8; 8] = [104, 104, 131, 86, 161, 189, 180, 216];

pub fn execute_payment(
    ctx: Context<ExecutePayment>,
    pay_mint: Pubkey,
    pay_amount: u64,
    in_index: u8,
    out_index: u8,
    min_out: u64,
) -> Result<()> {
    // 1. Find discount
    let config = &ctx.accounts.config;
    let discount_entry = config.discounts.iter()
        .find(|d| d.mint == pay_mint)
        .ok_or(CustomError::UnsupportedToken)?;
    let discount_bps = discount_entry.discount_bps;

    require!(ctx.accounts.payment.status == PaymentStatus::Initialized, CustomError::InvalidState);

    // 2. move tokens from buyer to vault_in
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_source.to_account_info(),
                to: ctx.accounts.vault_in.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        pay_amount,
    )?;

    // 3. if mint != USD*, swap via internal CPI
    if pay_mint != ctx.accounts.usd_star_mint.key() {
        let binding = ctx.accounts.payment.key();
        let seeds: &[&[u8]] = &[
            b"vault",
            binding.as_ref(),
            &[ctx.bumps.vault_in],
        ];
        token::approve(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                Approve {
                    to: ctx.accounts.vault_in.to_account_info(),
                    delegate: ctx.accounts.in_trader.to_account_info(),
                    authority: ctx.accounts.vault_in.to_account_info(),
                }, 
                &[seeds]),
                pay_amount,
        )?;

        let payload = SwapExactInHintlessData {
            in_index,
            out_index,
            exact_amount_in: pay_amount,
            min_amount_out: min_out,
        };
        let mut data = SWAP_EXACT_IN_DISCRIMINATOR.to_vec();
        data.extend(payload.try_to_vec()?);

        let ix = Instruction {
            program_id: *ctx.accounts.pool.owner,
            accounts: vec![
                AccountMeta::new(ctx.accounts.pool.key(), false),
                AccountMeta::new(ctx.accounts.in_mint.key(), false),
                AccountMeta::new(ctx.accounts.out_mint.key(), false),
                AccountMeta::new(ctx.accounts.in_trader.key(), false),
                AccountMeta::new(ctx.accounts.out_trader.key(), false),
                AccountMeta::new_readonly(ctx.accounts.numeraire_config.key(), false),
                AccountMeta::new(ctx.accounts.vault_in.key(), true), // signer (payer)
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false),
            ],
            data,
        };

        invoke_signed(
            &ix,
            &[
                ctx.accounts.pool.to_account_info(),
                ctx.accounts.in_mint.to_account_info(),
                ctx.accounts.out_mint.to_account_info(),
                ctx.accounts.in_trader.to_account_info(),
                ctx.accounts.out_trader.to_account_info(),
                ctx.accounts.numeraire_config.to_account_info(),
                ctx.accounts.vault_in.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
            &[seeds],
        )?;
    }

    // 4. record payment state
    let payment = &mut ctx.accounts.payment;
    payment.paid_mint = pay_mint;
    payment.paid_amount = pay_amount;
    payment.discount_bps = discount_bps;
    payment.status = PaymentStatus::Funded;
    
    Ok(())
}

#[derive(Accounts)]
pub struct ExecutePayment<'info> {
    #[account(mut)]
    pub payment: Account<'info, Payment>,

    pub config: Account<'info, GlobalConfig>,

    // Token vault that temporarily hodls buyer token(PDA)
    #[account(
        mut,
        seeds=[b"vault", payment.key().as_ref()],
        bump
    )]
    pub vault_in: Account<'info, TokenAccount>,

    pub usd_star_mint: Account<'info, Mint>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub buyer_source: Account<'info, TokenAccount>, // ATA for chosen token_mint

    // ---------- Numeraire pool & related (perena swap) ----------
    /// CHECK: pool
    pub pool: AccountInfo<'info>,
    /// CHECK: in_mint (same as pay_mint)
    pub in_mint: AccountInfo<'info>,
    /// CHECK: out_mint (USD*)
    pub out_mint: AccountInfo<'info>,

    #[account(mut)]
    pub in_trader: Account<'info, TokenAccount>,

    #[account(mut)]
    pub out_trader: Account<'info, TokenAccount>,

    /// CHECK: numeraire config
    pub numeraire_config: AccountInfo<'info>,
    /// CHECK: payer (== vault_in)
    pub payer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: token2022
    pub token_2022_program: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct SwapExactInHintlessData {
    in_index: u8,
    out_index: u8,
    exact_amount_in: u64,
    min_amount_out: u64,
}