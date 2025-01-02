use clap::{Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

use std::str::FromStr;

use spl_token::solana_program::pubkey::Pubkey;

use crate::{mint::MintCommands, token_account::TokenAccountCommands};

pub const DEFAULT_KEYPAIR_PATH: &str = ".config/solana/id.json";

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
    pub fn validate_args(&self) -> anyhow::Result<()> {
        self._check_rpc_url()?;
        Ok(())
    }

    fn _check_rpc_url(&self) -> anyhow::Result<()> {
        match &self.solana_rpc_url {
            Some(url) => {
                if url.starts_with("https://") {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Invalid RPC URL: {}", url))
                }
            }
            None => Err(anyhow::anyhow!(
                "RPC URL not provided use --solana-rpc-url or set `SOLANA_RPC_URL` env variable"
            )),
        }
    }

    pub fn parse_pubkey(pubkey_str: &str) -> anyhow::Result<Pubkey> {
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

#[derive(Debug, Clone)]
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

impl TryFrom<Pubkey> for TokenProgram {
    type Error = anyhow::Error;
    fn try_from(pubkey: Pubkey) -> anyhow::Result<Self> {
        match pubkey {
            spl_token::ID => Ok(TokenProgram::LegacyToken),
            spl_token_2022::ID => Ok(TokenProgram::Token2022),
            _ => Err(anyhow::anyhow!("Invalid token program pubkey: {}", pubkey)),
        }
    }
}

impl TokenProgram {
    pub fn _to_select_vec() -> Vec<&'static str> {
        vec!["Token2022", "LegacyToken"]
    }

    pub fn _from_select_str(select_str: &str) -> anyhow::Result<Self> {
        match select_str {
            "Token2022" => Ok(TokenProgram::Token2022),
            "LegacyToken" => Ok(TokenProgram::LegacyToken),
            _ => Err(anyhow::anyhow!("Invalid token program: {}", select_str)),
        }
    }
}

pub struct LocalWallet {
    pub keypair: Keypair,
}

impl LocalWallet {
    pub fn fetch() -> anyhow::Result<LocalWallet> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home dir"))?;
        let keypair = solana_sdk::signature::read_keypair_file(home_dir.join(DEFAULT_KEYPAIR_PATH))
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read keypair file from path {}: {}",
                    DEFAULT_KEYPAIR_PATH,
                    e
                )
            })?;
        Ok(LocalWallet { keypair })
    }

    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    pub async fn sign_and_send_ixs(
        &self,
        ixs: Vec<solana_sdk::instruction::Instruction>,
        rpc_url: &str,
    ) -> anyhow::Result<Signature> {
        let mut tx = solana_sdk::transaction::Transaction::new_with_payer(
            &ixs,
            Some(&self.keypair.pubkey()),
        );
        let rpc_client = RpcClient::new(rpc_url.to_string());
        let recent_blockhash = rpc_client.get_latest_blockhash().await?;
        tx.sign(&[&self.keypair], recent_blockhash);

        let sig = rpc_client.send_and_confirm_transaction(&tx).await?;

        println!("Transaction signature: {}", sig);

        Ok(sig)
    }
}
