use clap::{Args, Subcommand};
use prettytable::{color, Attr, Cell, Row};
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_token::{
    solana_program::{program_pack::Pack, pubkey::Pubkey},
    state::{Account, AccountState, Mint},
};

use crate::{
    cli::{self, SolanaRpcArgs},
    mint::{MintWithPubkey, PrettyMint},
};

#[derive(Debug)]
pub struct PrettyTokenAccount {
    pub token_account_pubkey: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub delegate: Option<String>,
    pub state: AccountState,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<String>,
}

pub struct TokenAccountWithPubkey {
    pub token_account: Account,
    pub pubkey: String,
}

impl PrettyTokenAccount {
    pub fn to_header_cell(header: &str) -> Cell {
        Cell::new(header)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::WHITE))
    }

    pub fn to_key_cell(key: &str) -> Cell {
        Cell::new(key)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::GREEN))
    }

    pub fn to_value_cell(value: &str) -> Cell {
        Cell::new(value)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN))
    }
    pub fn print(&self) {
        let mut table = prettytable::Table::new();

        table.add_row(Row::new(vec![Self::to_header_cell("Token Account Data")]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Token Account Pubkey"),
            Self::to_value_cell(&self.token_account_pubkey),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Mint"),
            Self::to_value_cell(&self.mint),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Owner"),
            Self::to_value_cell(&self.owner),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Amount"),
            Self::to_value_cell(&self.amount.to_string()),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Delegate"),
            Self::to_value_cell(&self.delegate.clone().map_or("None".to_string(), |pk| pk)),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("State"),
            Self::to_value_cell(match self.state {
                AccountState::Frozen => "Frozen",
                AccountState::Initialized => "Initialized",
                AccountState::Uninitialized => "Uninitialized",
            }),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Is Native"),
            Self::to_value_cell(&self.is_native.map_or("None".to_string(), |v| v.to_string())),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Delegated Amount"),
            Self::to_value_cell(&self.delegated_amount.to_string()),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Close Authority"),
            Self::to_value_cell(
                &self
                    .close_authority
                    .clone()
                    .map_or("None".to_string(), |pk| pk),
            ),
        ]));

        println!();
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
        table.printstd();
        println!();
    }
}

impl From<TokenAccountWithPubkey> for PrettyTokenAccount {
    fn from(ta_with_pubkey: TokenAccountWithPubkey) -> Self {
        Self {
            token_account_pubkey: ta_with_pubkey.pubkey.to_string(),
            mint: ta_with_pubkey.token_account.mint.to_string(),
            owner: ta_with_pubkey.token_account.owner.to_string(),
            amount: ta_with_pubkey.token_account.amount,
            delegate: ta_with_pubkey
                .token_account
                .delegate
                .map(|pk| pk.to_string())
                .into(),
            state: ta_with_pubkey.token_account.state,
            is_native: ta_with_pubkey
                .token_account
                .is_native
                .map(|v| v.into())
                .into(),
            delegated_amount: ta_with_pubkey.token_account.delegated_amount,
            close_authority: ta_with_pubkey
                .token_account
                .close_authority
                .map(|pk| pk.to_string())
                .into(),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TokenAccountCommands {
    Fetch(FetchTokenAccount),
    #[clap(subcommand)]
    Ix(TokenAccountInstructions),
}

impl TokenAccountCommands {
    pub async fn process(&self) -> anyhow::Result<()> {
        match self {
            TokenAccountCommands::Fetch(f) => {
                let (token_account, mint) = f.process_fetch().await?;
                let token_account: PrettyTokenAccount = TokenAccountWithPubkey {
                    token_account,
                    pubkey: f.account_pubkey.to_string(),
                }
                .into();

                let mint: PrettyMint = MintWithPubkey {
                    mint,
                    pubkey: token_account.mint.to_string(),
                }
                .into();

                mint.print();
                token_account.print();
            }
            TokenAccountCommands::Ix(ix) => {
                println!("Ix command:{:?}", ix);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct FetchTokenAccount {
    #[clap(flatten)]
    pub solana: SolanaRpcArgs,
    #[clap(
        value_parser = cli::Cli::parse_pubkey,
        help = "The account address to fetch. Must be a valid base58 encoded pubkey."
    )]
    pub account_pubkey: Pubkey,
    #[clap(
        value_parser = cli::Cli::parse_pubkey,
        help = "The mint address to fetch. Must be a valid base58 encoded pubkey."
    )]
    pub mint_pubkey: Option<Pubkey>,
}

impl FetchTokenAccount {
    pub async fn process_fetch(&self) -> anyhow::Result<(Account, Mint)> {
        if let Some(mint_pubkey) = self.mint_pubkey {
            // Concurrently fetch the token account and mint
            let rpc_client = RpcClient::new(self.solana.solana_rpc_url.clone());
            let fetch_res = tokio::join!(
                rpc_client.get_account(&self.account_pubkey),
                rpc_client.get_account(&mint_pubkey)
            );

            let (token_acc, mint_acc) = (
                fetch_res
                    .0
                    .map_err(|e| anyhow::anyhow!("Error fetching account: {}", e))?,
                fetch_res
                    .1
                    .map_err(|e| anyhow::anyhow!("Error fetching mint: {}", e))?,
            );

            let token_account = Account::unpack(&token_acc.data)
                .map_err(|e| anyhow::anyhow!("Error unpacking account: {}", e))?;

            let mint = Mint::unpack(&mint_acc.data)
                .map_err(|e| anyhow::anyhow!("Error unpacking mint: {}", e))?;

            return Ok((token_account, mint));
        } else {
            // Sequentially fetch the token account and mint

            let rpc_client = RpcClient::new(self.solana.solana_rpc_url.clone());
            let token_acc = rpc_client.get_account(&self.account_pubkey).await?;

            let token_account = Account::unpack(&token_acc.data)
                .map_err(|e| anyhow::anyhow!("Error unpacking account: {}", e))?;

            let mint_acc = rpc_client.get_account(&token_account.mint).await?;

            let mint = Mint::unpack(&mint_acc.data)
                .map_err(|e| anyhow::anyhow!("Error unpacking mint: {}", e))?;

            Ok((token_account, mint))
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TokenAccountInstructions {
    Create,
}
