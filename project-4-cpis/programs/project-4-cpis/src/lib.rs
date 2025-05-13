#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    token_interface::{self, TokenInterface},
};

declare_id!("Eau8idrUiVQD3r9eEcqhxdhCZVdXAcxQQ4YWNFFnC42P");

#[program]
pub mod project_4_cpis {
    use anchor_spl::token_interface::{self, MintTo, TransferChecked};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let token_project = &mut ctx.accounts.token_project;
        token_project.mint_authority = ctx.accounts.mint_authority.key();
        Ok(())
    }

    pub fn create_mint(_ctx: Context<CreateMint>, _decimals: u8) -> Result<()> {
        // Anchor already initialized it with the mint:: constraint
        Ok(())
    }

    pub fn create_token_account(_ctx: Context<CreateTokenAccount>) -> Result<()> {
        // Anchor already initialized it with the token:: constraint
        Ok(())
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token_interface::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn transfer_tokens(ctx: Context<TokenTransfer>, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.from.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let decimals = ctx.accounts.mint.decimals;
        token_interface::transfer_checked(cpi_ctx, amount, decimals)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32,
        seeds = [b"token-project", mint_authority.key().as_ref()],
        bump,
    )]
    pub token_project: Account<'info, TokenProjectAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(
        init,
        payer = mint_authority,
        mint::decimals = 9,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = owner,
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub mint_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenProjectAccount {
    pub mint_authority: Pubkey,
}

#[derive(Accounts)]
pub struct TokenTransfer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(mut)]
    pub from: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(mut)]
    pub to: InterfaceAccount<'info, token_interface::TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
