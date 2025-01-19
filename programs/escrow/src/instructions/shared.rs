use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

pub fn transfer_tokens<'info>(
    authority: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    to: &InterfaceAccount<'info, TokenAccount>,
    from: &InterfaceAccount<'info, TokenAccount>,
) -> Result<()> {
    let transfer_account = TransferChecked {
        from: from.to_account_info(),
        to: to.to_account_info(),
        mint: mint.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(token_program.to_account_info(), transfer_account);

    transfer_checked(cpi_context, *amount, mint.decimals)
}
