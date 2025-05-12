use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("94L2mJxVu6ZMmHaGsCHRQ65Kk2mea6aTnwWjSdfSsmBC");

#[program]
mod journal {
    use super::*;

    pub fn create_journal_entry(
        ctx: Context<CreateEntry>,
        title: String,
        message: String,
    ) -> Result<()> {
        msg!("Journal Entry Created");
        msg!("Title: {}", title);
        msg!("Message: {}", message);

        let user_profile = &mut ctx.accounts.user_profile;
        let journal_entry = &mut ctx.accounts.journal_entry;

        // Update user profile with new entry count
        user_profile.entry_count = user_profile.entry_count.checked_add(1).unwrap();

        // Initialize journal entry
        journal_entry.owner = ctx.accounts.owner.key();
        journal_entry.title = title;
        journal_entry.message = message;
        journal_entry.entry_number = user_profile.entry_count;
        journal_entry.created_at = Clock::get()?.unix_timestamp;
        journal_entry.bump = *ctx.bumps.get("journal_entry").unwrap();

        Ok(())
    }

    pub fn update_journal_entry(
        ctx: Context<UpdateEntry>,
        new_title: String,
        new_message: String,
    ) -> Result<()> {
        msg!("Journal Entry Updated");
        msg!("New Title: {}", new_title);
        msg!("New Message: {}", new_message);

        let journal_entry = &mut ctx.accounts.journal_entry;
        journal_entry.title = new_title;
        journal_entry.message = new_message;
        journal_entry.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn delete_journal_entry(ctx: Context<DeleteEntry>) -> Result<()> {
        let user_profile = &mut ctx.accounts.user_profile;
        let journal_entry = &mut ctx.accounts.journal_entry;

        msg!("Journal entry {} deleted", journal_entry.entry_number);

        // Update user profile
        user_profile.entry_count = user_profile.entry_count.checked_sub(1).unwrap();

        Ok(())
    }
}

#[account]
pub struct UserProfile {
    pub owner: Pubkey,
    pub entry_count: u64,
}

#[account]
pub struct JournalEntryState {
    pub owner: Pubkey,
    pub entry_number: u64,
    pub title: String,
    pub message: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct CreateEntry<'info> {
    #[account(
        init,
        seeds = [b"user_profile", owner.key().as_ref()],
        bump,
        payer = owner,
        space = 8 + 32 + 8
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        init,
        seeds = [b"journal_entry", owner.key().as_ref(), &user_profile.entry_count.to_le_bytes()],
        bump,
        payer = owner,
        space = 8 + 32 + 8 + 4 + 100 + 4 + 1000 + 8 + 8 + 1
    )]
    pub journal_entry: Account<'info, JournalEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateEntry<'info> {
    #[account(
        mut,
        seeds = [b"journal_entry", owner.key().as_ref(), &journal_entry.entry_number.to_le_bytes()],
        bump = journal_entry.bump,
        realloc = 8 + 32 + 8 + 4 + 100 + 4 + 1000 + 8 + 8 + 1,
        realloc::payer = owner,
        realloc::zero = true,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DeleteEntry<'info> {
    #[account(
        mut,
        seeds = [b"user_profile", owner.key().as_ref()],
        bump,
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        mut,
        seeds = [b"journal_entry", owner.key().as_ref(), &journal_entry.entry_number.to_le_bytes()],
        bump = journal_entry.bump,
        close = owner,
    )]
    pub journal_entry: Account<'info, JournalEntryState>,

    #[account(mut)]
    pub owner: Signer<'info>,
}
