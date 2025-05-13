use anchor_lang::prelude::*;

declare_id!("Hirm3Q5vTf2bTc83NMEfe8hjpdKrw6tdwK7DrjHnGR9Q");

#[program]
pub mod project_5_capstone {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
