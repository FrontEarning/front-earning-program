use anchor_lang::prelude::*;
use crate::{states::config::*, CustomError};

pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_discount_usdc_bps: u16,
    new_discount_usdt_bps: u16,
    new_discount_usd_star_bps: u16,
    new_maturity_secs: i64,
) -> Result<()> {
    require!(new_discount_usdc_bps <= 10_000, CustomError::InvalidBasisPoint);
    require!(new_discount_usdt_bps <= 10_000, CustomError::InvalidBasisPoint);
    require!(new_discount_usd_star_bps <= 10_000, CustomError::InvalidBasisPoint);

    let config = &mut ctx.accounts.config;
    config.usdc_discount_bps = new_discount_usdc_bps;
    config.usdt_discount_bps = new_discount_usdt_bps;
    config.usd_star_discount_bps = new_discount_usd_star_bps;
    config.settle_wait_secs = new_maturity_secs;

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut, has_one = owner)]
    pub config: Account<'info, Config>,
    pub owner: Signer<'info>,
}
