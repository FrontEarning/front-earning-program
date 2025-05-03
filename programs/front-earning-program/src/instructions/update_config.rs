use anchor_lang::prelude::*;
use crate::states::config::*;

pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_discounts: Option<Vec<DiscountEntry>>,
    new_maturity: Option<i64>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    if let Some(d) = new_discounts { config.discounts = d; }
    if let Some(m) = new_maturity { config.maturity_period_secs = m; }
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut, has_one = owner)]
    pub config: Account<'info, GlobalConfig>,
    pub owner: Signer<'info>,
}
