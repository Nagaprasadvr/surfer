pub mod metadata;

use clap::{Args, Subcommand};
use colored::*;
use inquire::Select;
use metadata::TokenMetadata;
use prettytable::{color, Attr, Cell, Row, Table};
use solana_account::Account;
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_token::solana_program::{program_pack::Pack, pubkey::Pubkey};
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use std::time::Duration;

use crate::{
    cli::{self, SolanaRpcArgs, TokenProgram},
    extension::{mint_account_extensions_data_bytes, ExtensionData},
    utils,
};

#[derive(Debug, Clone)]
pub struct PrettyMint {
    pub mint_pubkey: String,
    pub mint_authority: Option<String>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: Option<String>,
    pub extensions: Option<Vec<ExtensionData>>,
    pub token_metadata: Option<TokenMetadata>,
}

pub struct MintWithPubkey {
    pub mint_data: MintWithExtensions,
    pub pubkey: String,
}

pub enum MintType {
    LegacyMint(spl_token::state::Mint),
    Mint2022(spl_token_2022::state::Mint),
}

pub struct MintWithExtensions {
    pub base: MintType,
    pub token_metadata: Option<TokenMetadata>,
    pub extensions: Option<Vec<ExtensionData>>,
}

impl MintWithExtensions {
    pub fn try_parse_mint_with_extensions(
        data: Account,
        token_metadata: Option<TokenMetadata>,
    ) -> anyhow::Result<Self> {
        let token_program = TokenProgram::try_from(data.owner)?;
        let data_bytes = data.data.as_slice();
        match token_program {
            TokenProgram::LegacyToken => {
                let mint = spl_token::state::Mint::unpack(data_bytes)
                    .map_err(|e| anyhow::anyhow!("Error unpacking mint: {}", e))?;

                Ok(Self {
                    base: MintType::LegacyMint(mint),
                    token_metadata,
                    extensions: None,
                })
            }
            TokenProgram::Token2022 => {
                let unpacked =
                    StateWithExtensions::<spl_token_2022::state::Mint>::unpack(data_bytes)?;
                let extension_types = unpacked.get_extension_types()?;
                let mut extension_data_vec: Vec<ExtensionData> =
                    Vec::with_capacity(extension_types.len());

                for extension in extension_types {
                    let extension_data = mint_account_extensions_data_bytes(&unpacked, extension)?;
                    extension_data_vec.push(ExtensionData::try_from((extension, extension_data))?);
                }

                Ok(Self {
                    base: MintType::Mint2022(unpacked.base),
                    extensions: Some(extension_data_vec),
                    token_metadata,
                })
            }
        }
    }
}

impl PrettyMint {
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
        let mut table = Table::new();

        table.add_row(Row::new(vec![Self::to_header_cell("Mint Data")]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Mint Pubkey"),
            Self::to_value_cell(&self.mint_pubkey),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Mint Authority"),
            Self::to_value_cell(
                &self
                    .mint_authority
                    .clone()
                    .map_or("None".to_string(), |pk| pk),
            ),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Supply"),
            Self::to_value_cell(&self.supply.to_string()),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Decimals"),
            Self::to_value_cell(&self.decimals.to_string()),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Is Initialized"),
            Self::to_value_cell(&self.is_initialized.to_string()),
        ]));

        table.add_row(Row::new(vec![
            Self::to_key_cell("Freeze Authority"),
            Self::to_value_cell(
                &self
                    .freeze_authority
                    .clone()
                    .map_or("None".to_string(), |pk| pk),
            ),
        ]));

        println!();
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
        table.printstd();

        match &self.extensions {
            Some(extensions) => {
                let data = format!("{:#?}", extensions).cyan();
                if !extensions.is_empty() {
                    println!();
                    println!(" {} : {}", "Mint Extensions".purple().bold(), data.bold());
                }
            }
            None => {}
        }

        match &self.token_metadata {
            Some(metadata) => {
                println!();
                let meta = metadata
                    .metadata
                    .as_ref()
                    .map_or("None".to_string(), |m| format!("{:#?}", m));

                let master_edition = metadata
                    .master_edition
                    .as_ref()
                    .map_or("None".to_string(), |me| format!("{:#?}", me));

                if meta != "None" {
                    println!(
                        " {} : {}",
                        "Token Metadata".purple().bold(),
                        meta.cyan().bold()
                    );
                }

                println!();

                if master_edition != "None" {
                    println!(
                        " {} : {}",
                        "Master Edition".purple().bold(),
                        master_edition.cyan().bold()
                    );
                }
            }
            None => {}
        }
        println!();
    }
}

impl From<MintWithPubkey> for PrettyMint {
    fn from(mint_with_pubkey: MintWithPubkey) -> Self {
        match mint_with_pubkey.mint_data.base {
            MintType::LegacyMint(mint) => Self {
                mint_pubkey: mint_with_pubkey.pubkey.to_string(),
                mint_authority: mint.mint_authority.map(|pk| pk.to_string()).into(),
                supply: mint.supply,
                decimals: mint.decimals,
                is_initialized: mint.is_initialized,
                freeze_authority: mint.freeze_authority.map(|pk| pk.to_string()).into(),
                extensions: None,
                token_metadata: mint_with_pubkey.mint_data.token_metadata,
            },
            MintType::Mint2022(mint) => Self {
                mint_pubkey: mint_with_pubkey.pubkey.to_string(),
                mint_authority: mint.mint_authority.map(|pk| pk.to_string()).into(),
                supply: mint.supply,
                decimals: mint.decimals,
                is_initialized: mint.is_initialized,
                freeze_authority: mint.freeze_authority.map(|pk| pk.to_string()).into(),
                extensions: mint_with_pubkey.mint_data.extensions,
                token_metadata: mint_with_pubkey.mint_data.token_metadata,
            },
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum MintCommands {
    Fetch(FetchMint),
    Ix,
}

impl MintCommands {
    pub async fn process(&self) -> anyhow::Result<()> {
        match self {
            MintCommands::Fetch(f) => {
                let spinner = utils::get_spinner()?;
                spinner.enable_steady_tick(Duration::from_millis(100));
                let mint: PrettyMint = MintWithPubkey {
                    mint_data: f.process_fetch_and_parse().await?,
                    pubkey: f.mint_pubkey.to_string(),
                }
                .into();
                spinner.finish_and_clear();

                mint.print();
            }
            MintCommands::Ix => {
                let ix = MintInstructions::from_select_str(
                    Select::new(
                        "Select an instruction to execute",
                        MintInstructions::to_select_vec(),
                    )
                    .prompt()?,
                )?;

                println!("{:?}", ix);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct FetchMint {
    #[clap(flatten)]
    pub solana: SolanaRpcArgs,
    #[clap(value_parser = cli::Cli::parse_pubkey,
    help = "The mint address to fetch. Must be a valid base58 encoded pubkey.")]
    pub mint_pubkey: Pubkey,
}

impl FetchMint {
    pub async fn process_fetch_and_parse(&self) -> anyhow::Result<MintWithExtensions> {
        let rpc_client = RpcClient::new(self.solana.solana_rpc_url.clone());
        let acc = rpc_client.get_account(&self.mint_pubkey).await?;
        let token_metadata = TokenMetadata::fetch_and_parse(self.mint_pubkey, &rpc_client).await;
        let mint_with_extensions =
            MintWithExtensions::try_parse_mint_with_extensions(acc, token_metadata)?;

        Ok(mint_with_extensions)
    }
}

#[derive(Debug, Subcommand)]
pub enum MintInstructions {
    InitializeMint,
    SetAuthority,
    MintTo,
    MintToChecked,
    InitilaizeMint2,
}

impl MintInstructions {
    pub fn to_select_vec() -> Vec<&'static str> {
        vec![
            "InitializeMint",
            "SetAuthority",
            "MintTo",
            "MintToChecked",
            "InitilaizeMint2",
        ]
    }

    pub fn from_select_str(select_str: &str) -> anyhow::Result<Self> {
        match select_str {
            "InitializeMint" => Ok(Self::InitializeMint),
            "SetAuthority" => Ok(Self::SetAuthority),
            "MintTo" => Ok(Self::MintTo),
            "MintToChecked" => Ok(Self::MintToChecked),
            "InitilaizeMint2" => Ok(Self::InitilaizeMint2),

            _ => Err(anyhow::anyhow!("Invalid mint instruction: {}", select_str)),
        }
    }
}
