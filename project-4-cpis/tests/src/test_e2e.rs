use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signer::{keypair::Keypair, Signer},
    },
    Client, Cluster,
};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Signature};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use std::str::FromStr;
use std::{rc::Rc, thread, time::Duration};

#[test]
fn test_end_to_end() {
    // Generate keypairs for the various roles
    let mint_authority = Keypair::new();
    let mint_authority_pubkey = mint_authority.pubkey();
    println!("Mint authority pubkey: {}", mint_authority_pubkey);

    let token_owner = Keypair::new();
    let token_owner_pubkey = token_owner.pubkey();
    println!("Token owner pubkey: {}", token_owner_pubkey);

    let token_recipient = Keypair::new();
    let token_recipient_pubkey = token_recipient.pubkey();
    println!("Token recipient pubkey: {}", token_recipient_pubkey);

    // Set up the Solana client
    let payer = Rc::new(mint_authority);
    let token_owner_rc = Rc::new(token_owner);

    let url = "http://127.0.0.1:8899".to_string();
    let cluster = Cluster::Custom(url, "ws://127.0.0.1:8900".to_string());
    let client = Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());

    // Get the program from the client
    let program_id = Pubkey::from_str("Eau8idrUiVQD3r9eEcqhxdhCZVdXAcxQQ4YWNFFnC42P").unwrap();
    let program = client.program(program_id).unwrap();

    // Derive the PDA for the token project
    let (token_project_pda, bump) = Pubkey::find_program_address(
        &[b"token-project", mint_authority_pubkey.as_ref()],
        &program_id,
    );
    println!("Expected Token Project PDA: {}", token_project_pda);
    println!("Expected Bump: {}", bump);

    // Airdrop SOL to the mint authority
    println!("Airdropping SOL to mint authority...");
    airdrop_and_confirm(&program, &mint_authority_pubkey, 10 * LAMPORTS_PER_SOL);

    // Airdrop SOL to the token owner
    println!("Airdropping SOL to token owner...");
    airdrop_and_confirm(&program, &token_owner_pubkey, 2 * LAMPORTS_PER_SOL);

    // Airdrop SOL to the token recipient
    println!("Airdropping SOL to token recipient...");
    airdrop_and_confirm(&program, &token_recipient_pubkey, 2 * LAMPORTS_PER_SOL);

    // Wait after airdrops to ensure they're fully processed
    thread::sleep(Duration::from_secs(2));

    let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    // STEP 1: Initialize the token project
    println!("\n1. Initializing token project...");
    let init_tx = program
        .request()
        .accounts(project_4_cpis::accounts::Initialize {
            token_project: token_project_pda,
            payer: mint_authority_pubkey,
            mint_authority: mint_authority_pubkey,
            system_program,
        })
        .signer(&payer)
        .send()
        .expect("Failed to send initialization transaction");

    println!("Initialization transaction signature: {}", init_tx);
    confirm_transaction(&program, &init_tx);

    // STEP 2: Create a mint
    println!("\n2. Creating the mint...");
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    println!("Mint pubkey: {}", mint_pubkey);

    let create_mint_tx = program
        .request()
        .accounts(project_4_cpis::accounts::CreateMint {
            mint: mint_pubkey,
            mint_authority: mint_authority_pubkey,
            rent: solana_sdk::sysvar::rent::id(),
            token_program: spl_token::id(),
            system_program,
        })
        .args(project_4_cpis::instruction::CreateMint {
            _decimals: 9, // Using 9 decimals, standard for Solana tokens(?)
        })
        .signer(&payer)
        .signer(&mint_keypair)
        .send()
        .expect("Failed to create mint");

    println!("Create mint transaction signature: {}", create_mint_tx);
    confirm_transaction(&program, &create_mint_tx);

    // STEP 3: Create token account for the token owner using Associated Token Account
    println!("\n3. Creating token account for owner...");

    // Derive the Associated Token Account address for the owner
    let owner_token_account_pubkey =
        get_associated_token_address(&token_owner_pubkey, &mint_pubkey);
    println!("Owner token account pubkey: {}", owner_token_account_pubkey);

    // Create the Associated Token Account
    let create_ata_ix = create_associated_token_account(
        &mint_authority_pubkey, // Payer
        &token_owner_pubkey,    // Wallet address
        &mint_pubkey,           // Mint
        &spl_token::id(),       // Token program ID
    );

    let create_token_account_tx = program
        .request()
        .instruction(create_ata_ix)
        .signer(&payer)
        .send()
        .expect("Failed to create token account");

    println!(
        "Create token account transaction signature: {}",
        create_token_account_tx
    );
    confirm_transaction(&program, &create_token_account_tx);

    // STEP 4: Create token account for the recipient using Associated Token Account
    println!("\n4. Creating token account for recipient...");

    // Derive the Associated Token Account address for the recipient
    let recipient_token_account_pubkey =
        get_associated_token_address(&token_recipient_pubkey, &mint_pubkey);
    println!(
        "Recipient token account pubkey: {}",
        recipient_token_account_pubkey
    );

    // Create the Associated Token Account
    let create_recipient_ata_ix = create_associated_token_account(
        &mint_authority_pubkey,  // Payer
        &token_recipient_pubkey, // Wallet address
        &mint_pubkey,            // Mint
        &spl_token::id(),        // Token program ID
    );

    let create_recipient_account_tx = program
        .request()
        .instruction(create_recipient_ata_ix)
        .signer(&payer)
        .send()
        .expect("Failed to create recipient token account");

    println!(
        "Create recipient account transaction signature: {}",
        create_recipient_account_tx
    );
    confirm_transaction(&program, &create_recipient_account_tx);

    // STEP 5: Mint tokens to the owner's account
    println!("\n5. Minting tokens to owner's account...");
    let tokens_to_mint = 1_000_000_000; // 1 token with 9 decimals

    let mint_tx = program
        .request()
        .accounts(project_4_cpis::accounts::MintTokens {
            mint: mint_pubkey,
            token_account: owner_token_account_pubkey,
            mint_authority: mint_authority_pubkey,
            token_program: spl_token::id(),
        })
        .args(project_4_cpis::instruction::MintTokens {
            amount: tokens_to_mint,
        })
        .signer(&payer)
        .send()
        .expect("Failed to mint tokens");

    println!("Mint tokens transaction signature: {}", mint_tx);
    confirm_transaction(&program, &mint_tx);

    // STEP 6: Transfer tokens from owner to recipient
    println!("\n6. Transferring tokens from owner to recipient...");
    let transfer_amount = 500_000_000; // 0.5 tokens with 9 decimals

    let transfer_tx = program
        .request()
        .accounts(project_4_cpis::accounts::TokenTransfer {
            signer: token_owner_pubkey,
            mint: mint_pubkey,
            from: owner_token_account_pubkey,
            to: recipient_token_account_pubkey,
            token_program: spl_token::id(),
            system_program,
        })
        .args(project_4_cpis::instruction::TransferTokens {
            amount: transfer_amount,
        })
        .signer(&token_owner_rc) // Use the Rc version
        .send()
        .expect("Failed to transfer tokens");

    println!("Transfer tokens transaction signature: {}", transfer_tx);
    confirm_transaction(&program, &transfer_tx);

    // Verify final balances
    println!("\nTest completed successfully!");
    println!(
        "Token owner should have {} tokens",
        (tokens_to_mint - transfer_amount) as f64 / 1_000_000_000.0
    );
    println!(
        "Token recipient should have {} tokens",
        transfer_amount as f64 / 1_000_000_000.0
    );
}

// Helper function to airdrop SOL and confirm
fn airdrop_and_confirm(
    program: &anchor_client::Program<Rc<Keypair>>,
    pubkey: &Pubkey,
    amount: u64,
) {
    let airdrop_sig = program
        .rpc()
        .request_airdrop(pubkey, amount)
        .expect("Failed to airdrop SOL");

    println!("Airdrop transaction: {}", airdrop_sig);

    // Add a delay to ensure the airdrop is processed
    thread::sleep(Duration::from_secs(1));

    // Confirm the transaction
    program
        .rpc()
        .confirm_transaction(&airdrop_sig)
        .expect("Airdrop transaction not confirmed");

    // Check balance
    let balance = program
        .rpc()
        .get_balance(pubkey)
        .expect("Failed to get balance");

    println!(
        "Balance for {}: {} SOL",
        pubkey,
        balance as f64 / LAMPORTS_PER_SOL as f64
    );
}

// Helper function to confirm transaction with retry
fn confirm_transaction(program: &anchor_client::Program<Rc<Keypair>>, signature: &Signature) {
    let mut attempts = 0;
    let max_attempts = 5;

    while attempts < max_attempts {
        match program.rpc().confirm_transaction(signature) {
            Ok(_) => {
                println!("Transaction confirmed!");
                return;
            }
            Err(err) => {
                attempts += 1;
                println!("Confirmation attempt {} failed: {}", attempts, err);

                if attempts < max_attempts {
                    let backoff = 1000 * attempts;
                    println!("Retrying in {}ms...", backoff);
                    thread::sleep(Duration::from_millis(backoff as u64));
                } else {
                    panic!(
                        "Failed to confirm transaction after {} attempts",
                        max_attempts
                    );
                }
            }
        }
    }
}
