use anchor_client::{Client, Cluster};
use anyhow::Result;
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::AccountMeta, instruction::Instruction, pubkey::Pubkey,
    signature::read_keypair_file, signer::Signer,
};
use std::{rc::Rc, str::FromStr};

// Program ID of the voting application
const VOTING_PROGRAM_ID: &str = "5Couhd2qWo7v3L8LR3Q4daDPdFyJpV8MNqi3wkzNWGvu";

// The poll ID you want to vote in
const POLL_ID: u64 = 1;

// The name of the candidate you want to vote for
const CANDIDATE_NAME: &str = "Smooth"; // Or "Crunchy"

// #[tokio::main]
fn main() -> Result<()> {
    let signer = read_keypair_file("/Users/joseph/.config/solana/id.json")
        .map_err(|e| anyhow::anyhow!("Error reading keypair file: {}", e))?;
    let signer_pub_key = signer.pubkey();
    let client = Client::new(Cluster::Devnet, Rc::new(signer));
    let program = client.program(Pubkey::from_str(VOTING_PROGRAM_ID)?)?;

    let poll_id = POLL_ID;
    let candidate = CANDIDATE_NAME;

    let (poll_account_pubkey, _poll_bump) = Pubkey::find_program_address(
        &[b"poll", &poll_id.to_le_bytes()],
        &Pubkey::from_str(VOTING_PROGRAM_ID)?,
    );
    println!("Poll Account: {}", poll_account_pubkey);

    let (candidate_account_pubkey, _candidate_bump) = Pubkey::find_program_address(
        &[&poll_id.to_le_bytes(), candidate.as_bytes()],
        &Pubkey::from_str(VOTING_PROGRAM_ID)?,
    );
    println!("Candidate Account: {candidate_account_pubkey}");

    let mut hasher = Sha256::new();
    hasher.update(b"global:vote");
    let discriminator = &hasher.finalize()[..8];

    let tx = program
        .request()
        .accounts(vec![
            AccountMeta::new(signer_pub_key, true),
            AccountMeta::new(poll_account_pubkey, false),
            AccountMeta::new(candidate_account_pubkey, false),
        ])
        .instruction(Instruction {
            program_id: program.id(),
            accounts: vec![
                AccountMeta::new(signer_pub_key, true),
                AccountMeta::new(poll_account_pubkey, false),
                AccountMeta::new(candidate_account_pubkey, false),
            ],
            data: vec![
                discriminator.to_vec(),
                candidate.as_bytes().to_vec(),
                poll_id.to_le_bytes().to_vec(),
            ]
            .concat(),
        })
        .send()?;

    println!("Vote cast successfully! Transaction signature: {}", tx);
    Ok(())
}
