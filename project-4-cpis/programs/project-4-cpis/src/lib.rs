#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("Eau8idrUiVQD3r9eEcqhxdhCZVdXAcxQQ4YWNFFnC42P");

#[program]
pub mod project_4_cpis {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", _ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub system_program: Program<'info, System>,
}
