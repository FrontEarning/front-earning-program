use anchor_lang::prelude::*;
use crate::states::{config::*, Payment, PaymentStatus};

pub fn initialize_payment(
    ctx: Context<InitializePayment>,
    price_usd: u64,
) -> Result<()> {
    let payment = &mut ctx.accounts.payment;
    let config = &ctx.accounts.config;

    payment.seller = ctx.accounts.seller.key();
    payment.buyer = ctx.accounts.buyer.key();
    payment.borrower = Pubkey::default();
    payment.price_usd = price_usd;
    payment.discount_bps = 0;
    payment.paid_mint = Pubkey::default();
    payment.paid_amount = 0;
    payment.usd_star_deposit = 0;
    payment.created_ts = Clock::get()?.unix_timestamp;
    payment.maturity_ts = payment.created_ts + config.maturity_period_secs;
    payment.status = PaymentStatus::Initialized;

    Ok(())
}

#[derive(Accounts)]
#[instruction(price_usd: u64)]
pub struct InitializePayment<'info> {
    #[account(
        init,
        payer = buyer,
        space = 8 + Payment::LEN,
        seeds = [b"payment", buyer.key().as_ref(), seller.key().as_ref(), price_usd.to_le_bytes().as_ref()],
        bump
    )]
    pub payment: Account<'info, Payment>,
    pub config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: seller signs nothing here
    pub seller: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}