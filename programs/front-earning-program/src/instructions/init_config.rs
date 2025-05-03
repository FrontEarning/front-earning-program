use anchor_lang::prelude::*;
use crate::states::config::*;
use crate::error::CustomError;

pub fn init_config(
    ctx: Context<InitConfig>,
    discounts: Vec<DiscountEntry>,
    maturity_period_secs: i64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.owner = *ctx.accounts.owner.key;
    config.discounts = discounts;
    config.maturity_period_secs = maturity_period_secs;

    Ok(())
}

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(init, payer = owner, space = GlobalConfig::LEN_FIXED + 2000)]
    pub config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}