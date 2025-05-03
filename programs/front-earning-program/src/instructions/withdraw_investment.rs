use anchor_lang::prelude::*;

pub fn withdraw_investment(
    ctx: Context<WithdrawInvestment>,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawInvestment<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
}