use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
    },
    Client, Cluster,
};
use project_5_capstone::Store;
use std::{rc::Rc, str::FromStr, thread, time::Duration};

#[test]
fn test_register_store_and_initialize_config() {
    // Generate keypairs for store authority
    let store_authority = Rc::new(Keypair::new());
    let store_authority_pubkey = store_authority.pubkey();

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
    let program_id = Pubkey::from_str("Hirm3Q5vTf2bTc83NMEfe8hjpdKrw6tdwK7DrjHnGR9Q").unwrap();
    let program = client.program(program_id).unwrap();

    // Derive the PDA for the store
    let (store_pda, _store_bump) =
        Pubkey::find_program_address(&[b"store", store_authority_pubkey.as_ref()], &program_id);
    println!("Expected Store PDA: {}", store_pda);

    // Derive the PDA for the config
    let (config_pda, _config_bump) = Pubkey::find_program_address(&[b"config"], &program_id);
    println!("Expected Config PDA: {}", config_pda);

    // Airdrop SOL to the store authority
    airdrop_and_confirm(&program, &store_authority_pubkey, 2_000_000_000); // 2 SOL
    thread::sleep(Duration::from_secs(2));

    // Register the store
    let initial_store_name = "Initial Store".to_string();
    let register_store_tx = program
        .request()
        .accounts(project_5_capstone::accounts::RegisterStore {
            store: store_pda,
            authority: store_authority_pubkey,
            system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        })
        .args(project_5_capstone::instruction::RegisterStore {
            name: initial_store_name.clone(),
        })
        .signer(&*store_authority)
        .send()
        .expect("Failed to register store");
    println!(
        "Register store transaction signature: {}",
        register_store_tx
    );
    confirm_transaction(&program, &register_store_tx);

    // Initialize config and update store
    let new_store_name = "Updated Store".to_string();
    let initialize_config_tx = program
        .request()
        .accounts(
            project_5_capstone::accounts::InitializeConfigAndUpdateStore {
                store: store_pda,
                config: config_pda,
                authority: store_authority_pubkey,
                system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            },
        )
        .args(
            project_5_capstone::instruction::InitializeConfigAndUpdateStore {
                new_store_name: new_store_name.clone(),
            },
        )
        .signer(&*store_authority)
        .send()
        .expect("Failed to initialize config and update store");
    println!(
        "Initialize config and update store transaction signature: {}",
        initialize_config_tx
    );
    confirm_transaction(&program, &initialize_config_tx);

    // Verify the updated store name AFTER the update transaction
    let fetched_store: Store = program.account(store_pda).expect("Failed to fetch store");
    assert_eq!(fetched_store.name, "Updated Store");

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
