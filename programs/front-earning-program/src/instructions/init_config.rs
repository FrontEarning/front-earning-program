use anchor_lang::prelude::*;
use crate::states::config::*;

pub fn init_config(
    ctx: Context<InitConfig>,
    usd_star_discount_bps: u16,
    usdc_discount_bps: u16,
    usdt_discount_bps: u16,
    invest_lock_secs: i64,
    settle_wait_secs: i64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.owner = *ctx.accounts.owner.key;
    config.usd_star_discount_bps = usd_star_discount_bps;
    config.usdc_discount_bps = usdc_discount_bps;
    config.usdt_discount_bps = usdt_discount_bps;
    config.invest_lock_secs = invest_lock_secs;
    config.settle_wait_secs = settle_wait_secs;

    Ok(())
}

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(
        init, 
        payer = owner, 
        space = 8 + Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}