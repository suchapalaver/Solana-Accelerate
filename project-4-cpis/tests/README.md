# End-to-End Test Flow for the Solana Token Program

## Setup Phase

- **Creates key participants:**

  - Mint authority (creates and manages the token)
  - Token owner (will receive initial tokens)
  - Token recipient (will receive transferred tokens)

- **Establishes test environment:**

  - Sets up connection to local Solana validator
  - Calculates the expected Token Project PDA address
  - Airdrops SOL to all participants for transaction fees

## Token Creation & Setup Phase

1. **Initializes the token project**

   - Creates a PDA owned by the program to store token project details
   - Stores the mint authority's public key

2. **Creates the token mint**
   
- Creates a new SPL token with 9 decimals
- Sets the mint authority who controls token issuance

3. **Creates token accounts**

- Creates an Associated Token Account (ATA) for the token owner
- Creates an ATA for the token recipient
- Both accounts are linked to the specific mint and can hold the tokens

## Token Operations Phase

4. **Mints new tokens**

- Mints 1 token (1,000,000,000 base units with 9 decimals) 
- Deposits these tokens to the owner's token account

5. **Transfers tokens**

- Transfers 0.5 tokens from the owner to the recipient
- The token owner signs the transfer transaction

## Verification

- Prints the final expected token balances:
- Owner should have 0.5 tokens remaining
- Recipient should have received 0.5 tokens

## Helper Functions

- **airdrop_and_confirm**: Airdrops SOL and ensures transaction is confirmed
- **confirm_transaction**: Reliably confirms transactions with retries
