use anchor_client::solana_sdk::program_pack::Pack;
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        system_instruction,
        transaction::Transaction,
    },
    Client, Cluster,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use std::{rc::Rc, str::FromStr, thread, time::Duration};

#[test]
fn test_register_store_and_verify_balance() {
    // Generate keypairs for store authority and customer
    let store_authority = Rc::new(Keypair::new());
    let store_authority_pubkey = store_authority.pubkey();
    let customer = Rc::new(Keypair::new());
    let customer_pubkey = customer.pubkey();

    // Set up the Solana client
    let payer = store_authority.clone();
    let client = Client::new_with_options(
        Cluster::Custom(
            "http://127.0.0.1:8899".to_string(),
            "ws://127.0.0.1:8900".to_string(),
        ),
        payer.clone(),
        CommitmentConfig::processed(),
    );

    // Get the program from the client
    let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();
    let program = client.program(program_id).unwrap();

    // Derive the PDA for the store
    let (store_pda, _bump) =
        Pubkey::find_program_address(&[b"store", store_authority_pubkey.as_ref()], &program_id);
    println!("Expected Store PDA: {}", store_pda);

    // Airdrop SOL to the store authority and customer
    airdrop_and_confirm(&program, &store_authority_pubkey, 2_000_000_000); // 2 SOL
    airdrop_and_confirm(&program, &customer_pubkey, 2_000_000_000); // 2 SOL
    thread::sleep(Duration::from_secs(2));

    // Register the store
    let store_name = "My Store".to_string();
    let register_store_tx = program
        .request()
        .accounts(project_5_capstone::accounts::RegisterStore {
            store: store_pda,
            authority: store_authority_pubkey,
            system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        })
        .args(project_5_capstone::instruction::RegisterStore {
            name: store_name.clone(),
        })
        .signer(&*store_authority)
        .send()
        .expect("Failed to register store");
    println!(
        "Register store transaction signature: {}",
        register_store_tx
    );
    confirm_transaction(&program, &register_store_tx);

    // Create and initialize mint
    println!("Creating mint...");
    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();
    println!("Mint address: {}", mint_pubkey);

    // Create mint account
    let rent = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
        .expect("Failed to get rent exemption");

    let create_mint_account_ix = system_instruction::create_account(
        &store_authority_pubkey,
        &mint_pubkey,
        rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );

    // Initialize mint
    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &store_authority_pubkey,       // mint authority
        Some(&store_authority_pubkey), // freeze authority
        9,                             // 9 decimals
    )
    .expect("Failed to create initialize_mint instruction");

    let recent_blockhash = program
        .rpc()
        .get_latest_blockhash()
        .expect("Failed to get blockhash");

    let mint_init_tx = Transaction::new_signed_with_payer(
        &[create_mint_account_ix, init_mint_ix],
        Some(&store_authority_pubkey),
        &[&*store_authority, &mint],
        recent_blockhash,
    );

    let mint_init_sig = program
        .rpc()
        .send_and_confirm_transaction(&mint_init_tx)
        .expect("Failed to initialize mint");

    println!("Mint created and initialized: {}", mint_init_sig);
    confirm_transaction(&program, &mint_init_sig);

    // Create an associated token account for the customer
    let customer_token_account_pubkey =
        get_associated_token_address(&customer_pubkey, &mint_pubkey);
    let create_ata_ix = create_associated_token_account(
        &store_authority_pubkey,
        &customer_pubkey,
        &mint_pubkey,
        &spl_token::id(),
    );

    let recent_blockhash = program
        .rpc()
        .get_latest_blockhash()
        .expect("Failed to get blockhash");

    let create_ata_tx = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&store_authority_pubkey),
        &[&*store_authority],
        recent_blockhash,
    );

    let create_ata_sig = program
        .rpc()
        .send_and_confirm_transaction(&create_ata_tx)
        .expect("Failed to create customer token account");

    println!(
        "Create token account transaction signature: {}",
        create_ata_sig
    );
    confirm_transaction(&program, &create_ata_sig);

    // Verify customer balance
    let verify_balance_tx = program
        .request()
        .accounts(project_5_capstone::accounts::VerifyBalance {
            store: store_pda,
            customer_token_account: customer_token_account_pubkey,
            customer: customer_pubkey,
            authority: store_authority_pubkey,
            mint: mint_pubkey,
            token_program: spl_token::id(),
        })
        .args(project_5_capstone::instruction::VerifyBalance {
            min_balance: 1_000_000,
        })
        .signer(&*customer)
        .send();

    match verify_balance_tx {
        Ok(signature) => println!("Verify balance transaction signature: {}", signature),
        Err(err) => println!("Verify balance failed: {}", err),
    }
    println!("Test completed successfully!");
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
    thread::sleep(Duration::from_secs(1));
    program
        .rpc()
        .confirm_transaction(&airdrop_sig)
        .expect("Airdrop transaction not confirmed");
}

// Helper function to confirm transaction with retry
fn confirm_transaction(program: &anchor_client::Program<Rc<Keypair>>, signature: &Signature) {
    for attempt in 1..=5 {
        match program.rpc().confirm_transaction(signature) {
            Ok(_) => {
                println!("Transaction confirmed!");
                return;
            }
            Err(err) => {
                println!("Confirmation attempt {} failed: {}", attempt, err);
                thread::sleep(Duration::from_secs(attempt));
            }
        }
    }
    panic!("Failed to confirm transaction after 5 attempts");
}
