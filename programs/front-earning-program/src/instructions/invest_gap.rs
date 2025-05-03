use anchor_lang::prelude::*;

pub fn invest_gap(
    ctx: Context<InvestGap>,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct InvestGap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
}