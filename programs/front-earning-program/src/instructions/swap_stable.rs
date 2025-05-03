use anchor_lang::{prelude::*, solana_program::{instruction::Instruction, program::invoke_signed}};
use anchor_spl::token::{self, Token, TokenAccount};
use crate::{states::{deal::*, Payment}, CustomError};

const SWAP_EXACT_IN_DISCRIMINATOR: [u8; 8] = [104, 104, 131, 86, 161, 189, 180, 216];

#[derive(AnchorSerialize, AnchorDeserialize)]
struct SwapExactInHintlessData {
    in_index: u8,
    out_index: u8,
    exact_amount_in: u64,
    min_amount_out: u64,
}

pub fn swap_stable(
    ctx: Context<SwapStable>, 
    in_index: u8, 
    out_index: u8, 
    amount_in: u64, 
    min_out: u64) -> Result<()> {
    require!(amount_in > 0, CustomError::InputTooSmall);

    // 1. approve pool to pull token
    let binding = ctx.accounts.payment.key();
    let signer_seeds: &[&[u8]] = &[
        b"vault",
        binding.as_ref(),
        &[ctx.bumps.vault_in],
    ];

    token::approve(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Approve {
                to: ctx.accounts.vault_in.to_account_info(),
                delegate: ctx.accounts.payer.to_account_info(),
                authority: ctx.accounts.vault_in.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount_in,
    )?;

    // 2. build CPI ix
    let payload = SwapExactInHintlessData {
        in_index,
        out_index,
        exact_amount_in: amount_in,
        min_amount_out: min_out,
    };
    let mut ix_data = SWAP_EXACT_IN_DISCRIMINATOR.to_vec();
    ix_data.extend(payload.try_to_vec()?);

    let mut accounts = vec![
        AccountMeta::new(ctx.accounts.pool.key(),        false),
        AccountMeta::new(ctx.accounts.in_mint.key(),     false),
        AccountMeta::new(ctx.accounts.out_mint.key(),    false),
        AccountMeta::new(ctx.accounts.in_trader.key(),   false),
        AccountMeta::new(ctx.accounts.out_trader.key(),  false),
    ];

    if let Some(v) = ctx.accounts.in_vault.as_ref() {
        accounts.push(AccountMeta::new(v.key(), false));
    }
    if let Some(v) = ctx.accounts.out_vault.as_ref() {
        accounts.push(AccountMeta::new(v.key(), false));
    }

    accounts.extend([
        AccountMeta::new_readonly(ctx.accounts.numeraire_config.key(), false),
        AccountMeta::new(ctx.accounts.payer.key(),                     true), // signer
        AccountMeta::new_readonly(ctx.accounts.token_program.key(),    false),
        AccountMeta::new_readonly(ctx.accounts.token_2022_program.key(), false),
    ]);

    let ix = Instruction {
        program_id: *ctx.accounts.pool.owner,
        accounts,
        data: ix_data,
    };

    // 3. execute CPI (invoke signed)
    invoke_signed(
        &ix,
        &[
            ctx.accounts.pool.to_account_info(),
            ctx.accounts.in_mint.to_account_info(),
            ctx.accounts.out_mint.to_account_info(),
            ctx.accounts.in_trader.to_account_info(),
            ctx.accounts.out_trader.to_account_info(),
            // Optionals
            ctx.accounts.in_vault.as_ref().map(|a| a.to_account_info()).unwrap(),
            ctx.accounts.out_vault.as_ref().map(|a| a.to_account_info()).unwrap(),
            ctx.accounts.numeraire_config.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_2022_program.to_account_info(),
        ],
        &[signer_seeds],
    )?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct SwapStable<'info> {
    /// Current deal
    #[account(mut)]
    pub payment: Account<'info, Payment>,

    /// Source vault
    #[account(
        mut,
        seeds=[b"vault", payment.key().as_ref()],
        bump
    )]
    pub vault_in: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_out: Account<'info, TokenAccount>,

    // ---------- Numeraire AMM ----------
    /// CHECK: StablePool PDA
    pub pool: AccountInfo<'info>,
    /// CHECK: in_mint (same as pay_mint)
    pub in_mint: AccountInfo<'info>,
    /// CHECK: out_mint (USD*)
    pub out_mint: AccountInfo<'info>,

    #[account(mut)]
    pub in_trader: Account<'info, TokenAccount>,
    #[account(mut)]
    pub out_trader: Account<'info, TokenAccount>,

    #[account(mut)]
    pub in_vault: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub out_vault: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: NumeraireConfig PDA
    pub numeraire_config: AccountInfo<'info>,
    
    /// CHECK: signer PDA( == vault_in.key())
    pub payer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: Token‑2022 program
    pub token_2022_program: AccountInfo<'info>,
}