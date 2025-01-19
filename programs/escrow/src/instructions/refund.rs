use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{TransferChecked, Mint, TokenAccount, TokenInterface, transfer_checked, CloseAccount, close_account}
};

use crate::Offer;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub maker_token: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = maker_token,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
     #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = maker_token,
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
    vault_ata: InterfaceAccount<'info, TokenAccount>,
}

pub fn refund_maker(ctx: &Context<Refund>) -> Result<()> {
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"escrow",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.escrow.id.to_le_bytes()[..],
        &[ctx.accounts.escrow.bump],
    ]];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.vault_ata.to_account_info(),
        to: ctx.accounts.maker_token_ata.to_account_info(),
        mint: ctx.accounts.maker_token.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info()
    };
    
    let cpi = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), transfer_accounts, 
        &signer_seeds
    );

    transfer_checked(cpi, ctx.accounts.escrow.amount, ctx.accounts.maker_token.decimals)
}
pub fn close_escrow(ctx: Context<Refund>) -> Result<()> {
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"escrow",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.escrow.id.to_le_bytes()[..],
        &[ctx.accounts.escrow.bump],
    ]];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.vault_ata.to_account_info(),
        to: ctx.accounts.maker_token_ata.to_account_info(),
        mint: ctx.accounts.maker_token.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info()
    };

    let cpi_transfer = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts , &signer_seeds);

    transfer_checked(cpi_transfer, ctx.accounts.vault_ata.amount, ctx.accounts.maker_token.decimals);

    let close_accounts = CloseAccount {
        account: ctx.accounts.vault_ata.to_account_info(),
        destination: ctx.accounts.maker.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info()
    };

    let cpi_close = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), close_accounts, &signer_seeds);

    close_account(cpi_close)
}
