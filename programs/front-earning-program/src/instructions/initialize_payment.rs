use anchor_lang::prelude::*;
use crate::{states::{config::*, Payment, PaymentStatus}, CustomError};

pub fn initialize_payment(
    ctx: Context<InitializePayment>,
    price: u64,
) -> Result<()> {
    require!(price > 0, CustomError::InputTooSmall);
    let payment = &mut ctx.accounts.payment;

    payment.seller = ctx.accounts.seller.key();
    payment.discount_bps = 0; // initial discount
    payment.amount = price;
    payment.paid_amount = 0;
    payment.paid_mint = Pubkey::default();
    payment.status = PaymentStatus::Initialized;
    payment.gap_amount = 0;
    payment.gap_shares = 0;

    Ok(())
}

#[derive(Accounts)]
#[instruction(price: u64, maturity_ts: i64)]
pub struct InitializePayment<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = seller,
        space = 8 + Payment::LEN,
        seeds = [b"payment", seller.key().as_ref(), price.to_le_bytes().as_ref()],
        bump
    )]
    pub payment: Account<'info, Payment>,

    pub system_program: Program<'info, System>,
}