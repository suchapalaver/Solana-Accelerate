use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod project_5_capstone {
    use super::*;

    pub fn register_store(ctx: Context<RegisterStore>, name: String) -> Result<()> {
        let store = &mut ctx.accounts.store;
        store.name = name;
        store.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn verify_balance(ctx: Context<VerifyBalance>, min_balance: u64) -> Result<()> {
        let token_account = &ctx.accounts.customer_token_account;
        if token_account.amount < min_balance {
            return Err(error!(ErrorCode::InsufficientBalance));
        }
        Ok(())
    }

    pub fn initialize_config_and_update_store(
        ctx: Context<InitializeConfigAndUpdateStore>,
        new_store_name: String,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.owner = ctx.accounts.authority.key();

        let store = &mut ctx.accounts.store;
        store.name = new_store_name;

        Ok(())
    }
}

#[account]
pub struct Store {
    pub name: String,
    pub authority: Pubkey,
}

#[account]
pub struct TokenProjectAccount {
    pub mint_authority: Pubkey,
}

#[account]
pub struct Config {
    pub owner: Pubkey,
}

#[derive(Accounts)]
pub struct RegisterStore<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 40,
        seeds = [b"store", authority.key().as_ref()],
        bump
    )]
    pub store: Account<'info, Store>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyBalance<'info> {
    #[account(mut, has_one = authority)]
    pub store: Account<'info, Store>,
    #[account(
        constraint = customer_token_account.owner == customer.key(),
        constraint = customer_token_account.mint == mint.key()
    )]
    pub customer_token_account: Account<'info, TokenAccount>,
    pub customer: Signer<'info>,
    pub authority: Signer<'info>,
    /// CHECK: The mint account is not read or written to directly in this instruction.
    pub mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32,
        seeds = [b"token-project"],
        bump
    )]
    pub token_project: Account<'info, TokenProjectAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeConfigAndUpdateStore<'info> {
    #[account(
        mut,
        seeds = [b"store", authority.key().as_ref()],
        bump
    )]
    pub store: Account<'info, Store>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Customer balance is insufficient.")]
    InsufficientBalance,
}
