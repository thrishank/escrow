use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub id: u64,
    pub maker: Pubkey,
    pub maker_token: Pubkey,
    pub taker_token: Pubkey,
    pub amount: u64,
    pub bump: u8,
}
