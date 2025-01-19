pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2AR2sfypTGcRaNh6CU3qehhduZxUKjUMoAGk1Zpggoa4");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, id: u64, deposit: u64, receive: u64) -> Result<()> {
        instructions::make::send_tokens_to_vault(&ctx, deposit);
        instructions::make::create_offer(ctx, id, receive);
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::send_tokens_maker(&ctx);
        instructions::take::withdraw_close_escrow(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::refund_maker(&ctx);
        instructions::refund::close_escrow(ctx)
    }
}
