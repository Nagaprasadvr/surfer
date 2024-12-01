use clap::{Args, Subcommand};
use colored::*;
use prettytable::{color, Attr, Cell, Row};
use solana_account::Account;
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_token::solana_program::{program_pack::Pack, pubkey::Pubkey};
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

use crate::{
    cli::{self, SolanaRpcArgs, TokenProgram},
    extension::{token_account_extensions_data_bytes, ExtensionData},
    mint::{MintWithExtensions, MintWithPubkey, PrettyMint},
};

#[derive(Debug)]
pub struct PrettyTokenAccount {
    pub token_account_pubkey: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub delegate: Option<String>,
    pub state: u8,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<String>,
    pub extensions: Option<Vec<ExtensionData>>,
}

pub struct TokenAccountWithPubkey {
    pub token_account: TokenAccountWithExtensions,
    pub pubkey: String,
}

pub enum TokenAccountType {
    LegacyToken(spl_token::state::Account),
    Token2022(spl_token_2022::state::Account),
}
pub struct TokenAccountWithExtensions {
    pub base: TokenAccountType,
    pub extensions: Option<Vec<ExtensionData>>,
}

impl TokenAccountWithExtensions {
    pub fn try_parse_token_account_with_extensions(data: Account) -> anyhow::Result<Self> {
        let token_program = TokenProgram::try_from(data.owner)?;

        let data_bytes = data.data.as_slice();

        match token_program {
            TokenProgram::LegacyToken => {
                let token_account = spl_token::state::Account::unpack(data_bytes)
                    .map_err(|e| anyhow::anyhow!("Error unpacking token account: {}", e))?;

                Ok(Self {
                    base: TokenAccountType::LegacyToken(token_account),
                    extensions: None,
                })
            }
            TokenProgram::Token2022 => {
                let unpacked =
                    StateWithExtensions::<spl_token_2022::state::Account>::unpack(data_bytes)?;
                let extension_types = unpacked.get_extension_types()?;
                let mut extension_data_vec: Vec<ExtensionData> =
                    Vec::with_capacity(extension_types.len());

                for extension in extension_types {
                    let extension_data = token_account_extensions_data_bytes(&unpacked, extension)?;
                    extension_data_vec.push(ExtensionData::try_from((extension, extension_data))?);
                }
                Ok(Self {
                    base: TokenAccountType::Token2022(unpacked.base),
                    extensions: Some(extension_data_vec),
                })
            }
        }
    }
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
                0 => "Frozen",
                1 => "Initialized",
                2 => "Uninitialized",
                _ => "Unknown",
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
        match &self.extensions {
            Some(extensions) => {
                let data = format!("{:#?}", extensions).bright_yellow();
                println!(
                    " {} : {}",
                    "Token Account Extensions".purple().bold(),
                    data.cyan().bold()
                );
            }
            None => {}
        }
        println!();
    }
}

impl From<TokenAccountWithPubkey> for PrettyTokenAccount {
    fn from(ta_with_pubkey: TokenAccountWithPubkey) -> Self {
        match ta_with_pubkey.token_account.base {
            TokenAccountType::LegacyToken(token_account) => Self {
                token_account_pubkey: ta_with_pubkey.pubkey.to_string(),
                mint: token_account.mint.to_string(),
                owner: token_account.owner.to_string(),
                amount: token_account.amount,
                delegate: token_account.delegate.map(|pk| pk.to_string()).into(),
                state: token_account.state as u8,
                is_native: token_account.is_native.map(|v| v.into()).into(),
                delegated_amount: token_account.delegated_amount,
                close_authority: token_account
                    .close_authority
                    .map(|pk| pk.to_string())
                    .into(),
                extensions: None,
            },
            TokenAccountType::Token2022(token_account) => Self {
                token_account_pubkey: ta_with_pubkey.pubkey.to_string(),
                mint: token_account.mint.to_string(),
                owner: token_account.owner.to_string(),
                amount: token_account.amount,
                delegate: token_account.delegate.map(|pk| pk.to_string()).into(),
                state: token_account.state as u8,
                is_native: token_account.is_native.map(|v| v.into()).into(),
                delegated_amount: token_account.delegated_amount,
                close_authority: token_account
                    .close_authority
                    .map(|pk| pk.to_string())
                    .into(),
                extensions: ta_with_pubkey.token_account.extensions,
            },
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

                let token_acc_data =
                    TokenAccountWithExtensions::try_parse_token_account_with_extensions(
                        token_account,
                    )?;

                let token_account: PrettyTokenAccount = TokenAccountWithPubkey {
                    token_account: token_acc_data,
                    pubkey: f.account_pubkey.to_string(),
                }
                .into();

                let mint_acc_data = MintWithExtensions::try_parse_mint_with_extensions(mint)?;

                let mint: PrettyMint = MintWithPubkey {
                    mint_data: mint_acc_data,
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
    pub async fn process_fetch(&self) -> anyhow::Result<(Account, Account)> {
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

            return Ok((token_acc, mint_acc));
        } else {
            // Sequentially fetch the token account and mint
            let rpc_client = RpcClient::new(self.solana.solana_rpc_url.clone());
            let token_acc = rpc_client.get_account(&self.account_pubkey).await?;

            let token_program = TokenProgram::try_from(token_acc.owner)?;

            println!("Token program: {:?}", token_program);

            match token_program {
                TokenProgram::LegacyToken => {
                    let token_account =
                        spl_token::state::Account::unpack(&token_acc.data.as_slice())
                            .map_err(|e| anyhow::anyhow!("Error unpacking token account: {}", e))?;

                    let mint_acc = rpc_client.get_account(&token_account.mint).await?;

                    Ok((token_acc, mint_acc))
                }
                TokenProgram::Token2022 => {
                    let token_account =
                        StateWithExtensions::<spl_token_2022::state::Account>::unpack(
                            &token_acc.data.as_slice(),
                        )?;

                    let mint_acc = rpc_client.get_account(&token_account.base.mint).await?;

                    Ok((token_acc, mint_acc))
                }
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TokenAccountInstructions {
    Create,
}
