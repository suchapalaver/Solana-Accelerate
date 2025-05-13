use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signer::{keypair::Keypair, Signer},
    },
    Client, Cluster,
};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use std::str::FromStr;
use std::{rc::Rc, thread, time::Duration};

#[test]
fn test_initialize() {
    let mint_authority = Keypair::new();
    let mint_authority_pubkey = mint_authority.pubkey();

    let payer = Rc::new(mint_authority);
    let url = "http://127.0.0.1:8899".to_string();
    let cluster = Cluster::Custom(url, "ws://127.0.0.1:8900".to_string());

    let client = Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());

    let program_id = Pubkey::from_str("Eau8idrUiVQD3r9eEcqhxdhCZVdXAcxQQ4YWNFFnC42P").unwrap();
    let program = client.program(program_id).unwrap();

    // Derive the PDA for the token project
    let (token_project_pda, bump) = Pubkey::find_program_address(
        &[b"token-project", mint_authority_pubkey.as_ref()],
        &program_id,
    );

    println!("Expected PDA: {token_project_pda}");
    println!("Expected Bump: {bump}");

    // Airdrop some SOL to the mint authority for transaction fees
    let airdrop_sig = program
        .rpc()
        .request_airdrop(&mint_authority_pubkey, 2 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL");
    println!("Airdrop transaction: {airdrop_sig}");

    // Add a delay to ensure the airdrop is fully processed - I find this is needed for this test to pass
    // TODO: Find a way to wait for the airdrop to be processed
    thread::sleep(Duration::from_secs(2));

    // Wait for the airdrop to confirm
    program
        .rpc()
        .confirm_transaction(&airdrop_sig)
        .expect("Airdrop transaction not confirmed");

    let tx = program
        .request()
        .accounts(project_4_cpis::accounts::Initialize {
            token_project: token_project_pda,
            payer: mint_authority_pubkey,
            mint_authority: mint_authority_pubkey,
            system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        })
        .send()
        .expect("Failed to send initialization transaction");

    println!("Initialization transaction signature: {tx}");
}
