use anyhow::Result;
use clap::{Parser, Subcommand};
use std::str::FromStr;

use spl_token::solana_program::pubkey::Pubkey;

use crate::{mint::MintCommands, token_account::TokenAccountCommands};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    /// Log level: trace, debug, info, warn, error, off
    #[clap(short, long, global = true)]
    pub log_level: Option<String>,

    /// RPC URL for the Solana cluster
    #[clap(short, long, env = "SOLANA_RPC_URL", global = true)]
    pub solana_rpc_url: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn validate_args(&self) -> Result<()> {
        self._check_rpc_url()?;
        Ok(())
    }

    fn _check_rpc_url(&self) -> Result<()> {
        match &self.solana_rpc_url {
            Some(url) => {
                if url.starts_with("https://") {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Invalid RPC URL: {}", url))
                }
            }
            None => Err(anyhow::anyhow!(
                "RPC URL not provided use --solana-rpc-url or set SOLANA_RPC_URL env variable"
            )),
        }
    }

    pub fn parse_pubkey(pubkey_str: &str) -> Result<Pubkey> {
        match Pubkey::from_str(pubkey_str) {
            Ok(pubkey) => Ok(pubkey),
            Err(err) => Err(anyhow::anyhow!("Invalid pubkey: {}", err)),
        }
    }
}
#[derive(Clone, Parser, Debug)]
pub struct SolanaRpcArgs {
    #[arg(long, short, env)]
    pub solana_rpc_url: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(subcommand)]
    Mint(MintCommands),
    #[clap(subcommand)]
    TokenAccount(TokenAccountCommands),
}

#[allow(dead_code)]
pub enum TokenProgram {
    Token2022,
    LegacyToken,
}

impl Into<Pubkey> for TokenProgram {
    fn into(self) -> Pubkey {
        match self {
            TokenProgram::Token2022 => spl_token::ID,
            TokenProgram::LegacyToken => spl_token_2022::ID,
        }
    }
}
