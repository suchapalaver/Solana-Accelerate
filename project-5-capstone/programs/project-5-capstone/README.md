# 📦 Project 5 Capstone - Solana Store Credit Verification

This is a Solana smart contract that allows multiple users (e.g., stores) to verify
if a customer holds a minimum token balance before granting access to special offers
or discounts.

It’s designed to be lightweight and extensible, making it easy to integrate with various token-based loyalty systems.

## 📝 Features

### Store Registration

Stores can register with the contract, creating a unique PDA for each store.

### Token Balance Verification

Verify customer token balances using the Solana Token Program.

### Security Checks

Uses PDAs to securely manage store data.

### CPI Integration

Supports cross-program invocations for token checks.
