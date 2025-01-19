use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, CloseAccount, close_account}
};

use crate::Offer;

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    
    pub maker_token: InterfaceAccount<'info, Mint>, 
    pub taker_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
  
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = maker_token, 
        has_one = taker_token,
        seeds = [b"escrow", maker.key().as_ref(), escrow.id.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Offer>,

    #[account(
        mut, 
        associated_token::mint = maker_token,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
     vault_mint_a: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = taker_token,
        associated_token::authority = maker,
        associated_token::token_program = token_program 
    )]
    pub maker_token_ata_mint_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_token,
        associated_token::authority = taker,
        associated_token::token_program = token_program 
    )]
    pub taker_token_ata_mint_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = taker_token,
        associated_token::authority = taker,
        associated_token::token_program = token_program 
    )]
    pub taker_token_ata_mint_b: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>
}


pub fn send_tokens_maker(ctx: &Context<Take>) -> Result<()> { 
    let transfer_accounts = TransferChecked {
        from: ctx.accounts.taker_token_ata_mint_b.to_account_info(),
        to: ctx.accounts.maker_token_ata_mint_b.to_account_info(),
        mint: ctx.accounts.taker_token.to_account_info(), 
        authority: ctx.accounts.taker.to_account_info()
    };

    let cpi_transfer = CpiContext::new(
        ctx.accounts.token_program.to_account_info(), 
        transfer_accounts
    );
    transfer_checked(cpi_transfer, ctx.accounts.escrow.amount, ctx.accounts.taker_token.decimals)
}

pub fn withdraw_close_escrow(ctx: Context<Take>) -> Result<()> {
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"escrow",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.escrow.id.to_le_bytes()[..],
        &[ctx.accounts.escrow.bump],
    ]];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.vault_mint_a.to_account_info(),
        to: ctx.accounts.taker_token_ata_mint_a.to_account_info(),
        mint: ctx.accounts.maker_token.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info()
    };

    let cpi = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), transfer_accounts, 
        &signer_seeds
    );

   transfer_checked(cpi, ctx.accounts.escrow.amount, ctx.accounts.maker_token.decimals); 

    let accounts = CloseAccount {
        account: ctx.accounts.vault_mint_a.to_account_info(),
        destination: ctx.accounts.taker.to_account_info(),
        authority:  ctx.accounts.escrow.to_account_info() 
    };

    let cpi_close = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds
    );

    close_account(cpi_close)
}
