use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{TokenInterface,TransferChecked, transfer_checked, Mint, TokenAccount};

use crate::Offer;
use crate::ANCHOR_DISCRIMINATOR;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        space =  ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Offer>,
    #[account(
        init, 
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn create_offer(ctx: Context<Make>, id:u64, amount:u64) -> Result<()> {
    ctx.accounts.escrow.set_inner(Offer {
        id,
        maker: ctx.accounts.maker.key(),
        maker_token: ctx.accounts.maker_token_ata.key(),
        amount,
        taker_token: ctx.accounts.mint_b.key(),
        bump: ctx.bumps.escrow
    });
    Ok(())
}
pub fn send_tokens_to_vault(ctx: &Context<Make>, amount:u64) -> Result<()> { 
    let transfer_accounts = TransferChecked {
        from: ctx.accounts.maker_token_ata.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        mint: ctx.accounts.mint_a.to_account_info(),
        authority: ctx.accounts.maker.to_account_info()
    };
    let cpi_transfer = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts);

    transfer_checked(cpi_transfer, amount, ctx.accounts.mint_a.decimals)
}
