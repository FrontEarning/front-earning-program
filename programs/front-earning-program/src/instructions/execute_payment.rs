use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::states::{config::*, payment::*};
use crate::error::CustomError;
use crate::swap_stable::{SwapStable, swap_stable};

pub fn execute_payment(
    ctx: Context<ExecutePayment>,
    pay_mint: Pubkey,
    pay_amount: u64,
) -> Result<()> {
    // 1. Find discount
    let config = &ctx.accounts.config;
    let discount_entry = config.discounts.iter()
        .find(|d| d.mint == pay_mint)
        .ok_or(CustomError::UnsupportedToken)?;
    let discount_bps = discount_entry.discount_bps;

    let payment = &mut ctx.accounts.payment;
    require!(payment.status == PaymentStatus::Initialized, CustomError::InvalidState);

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
    // if pay_mint != ctx.accounts.usd_star_mint.key() {
    //     let swap_ctx = Context::new(
    //         &ctx.program_id,
    //         SwapStable {
    //             payment: unsafe { core::mem::transmute(&ctx.accounts.payment) },
    //             vault_in: unsafe { core::mem::transmute(&ctx.accounts.vault_in) },
    //             vault_out: unsafe { core::mem::transmute(&ctx.accounts.out_trader) },
    //             pool: ctx.accounts.pool.clone(),
    //             in_mint: ctx.accounts.in_mint.clone(),
    //             out_mint: ctx.accounts.out_mint.clone(),
    //             in_trader: unsafe { core::mem::transmute(&ctx.accounts.in_trader) },
    //             out_trader: unsafe { core::mem::transmute(&ctx.accounts.out_trader) },
    //             in_vault: None,
    //             out_vault: None,
    //             numeraire_config: ctx.accounts.numeraire_config.clone(),
    //             payer: ctx.accounts.payer.clone(),
    //             token_program: ctx.accounts.token_program.clone(),
    //             token_2022_program: ctx.accounts.token_2022_program.clone(),
    //         },
    //         vec![],
    //         &ctx.bumps,
    //     );

    //     swap_stable(
    //         swap_ctx,
    //         0, // in_index - assume mapping 0/1 for mvp
    //         1, // out_index
    //         pay_amount,
    //         0, // no slippage check in mvp
    //     )?;
    // }

    // 4. record payment state
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
    pub buyer_source: Account<'info, TokenAccount>,

    // ---------- Numeraire pool & related (perena swap) ----------
    /// CHECK: pool
    pub pool: AccountInfo<'info>,
    /// CHECK: in_mint (same as pay_mint)
    pub in_mint: AccountInfo<'info>,
    /// CHECK: out_mint (USD*)
    pub out_mint: AccountInfo<'info>,

    #[account(mut)]
    pub out_trader: Account<'info, TokenAccount>,
    #[account(mut)]
    pub in_trader: Account<'info, TokenAccount>,

    /// CHECK: numeraire config
    pub numeraire_config: AccountInfo<'info>,
    /// CHECK: payer (== vault_in)
    pub payer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: token2022
    pub token_2022_program: AccountInfo<'info>,
}
