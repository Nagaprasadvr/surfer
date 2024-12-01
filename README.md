## Surfer - A solana token cli rewrite which is simple and adds support for token extensions and parsing

## Features

- **Fetch Mint account**

  - parse `mint` account
  - parse mint `extensions`
  - fetch and parse `token metadata`, `master edition` if available

- **Fetch Token account**

  - parse `token account`
  - parse token account `extensions`

> **Note:** This is a work in progress and will be updated with more features

- Send Mint Ixs

- Send Token Ixs

## Setup

### Env vars

```bash
export SOLANA_RPC_URL="https://api.devnet.solana.com"
```

### Running the cli

```bash
cargo run
```
