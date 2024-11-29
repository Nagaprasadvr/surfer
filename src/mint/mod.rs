use clap::{Args, Subcommand};
use prettytable::{row, Table};
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_token::{
    solana_program::{program_pack::Pack, pubkey::Pubkey},
    state::Mint,
};

use crate::cli::{self, SolanaRpcArgs};

#[derive(Debug)]
pub struct PrettyMint {
    pub mint_pubkey: String,
    pub mint_authority: Option<String>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: Option<String>,
}

pub struct MintWithPubkey {
    pub mint: Mint,
    pub pubkey: String,
}

impl PrettyMint {
    pub fn print(&self) {
        let mut table = Table::new();

        table.add_row(row!["Mint Pubkey", self.mint_pubkey]);
        table.add_row(row![
            "Mint Authority",
            self.mint_authority
                .clone()
                .map_or("None".to_string(), |pk| pk)
        ]);
        table.add_row(row!["Supply", self.supply]);
        table.add_row(row!["Decimals", self.decimals]);
        table.add_row(row!["Is Initialized", self.is_initialized]);
        table.add_row(row![
            "Freeze Authority",
            self.freeze_authority
                .clone()
                .map_or("None".to_string(), |pk| pk)
        ]);

        println!();
        println!("Mint Data:");
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
        table.printstd();
        println!();
    }
}

impl From<MintWithPubkey> for PrettyMint {
    fn from(mint_with_pubkey: MintWithPubkey) -> Self {
        Self {
            mint_pubkey: mint_with_pubkey.pubkey.to_string(),
            mint_authority: mint_with_pubkey
                .mint
                .mint_authority
                .map(|pk| pk.to_string())
                .into(),
            supply: mint_with_pubkey.mint.supply,
            decimals: mint_with_pubkey.mint.decimals,
            is_initialized: mint_with_pubkey.mint.is_initialized,
            freeze_authority: mint_with_pubkey
                .mint
                .freeze_authority
                .map(|pk| pk.to_string())
                .into(),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum MintCommands {
    Fetch(FetchMint),
    #[clap(subcommand)]
    Ix(MintInstructions),
}

impl MintCommands {
    pub async fn process(&self) -> anyhow::Result<()> {
        match self {
            MintCommands::Fetch(f) => {
                let mint: PrettyMint = MintWithPubkey {
                    mint: f.process_fetch().await?,
                    pubkey: f.mint_pubkey.to_string(),
                }
                .into();

                mint.print();
            }
            MintCommands::Ix(ix) => {
                println!("Ix command:{:?}", ix);
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
    pub async fn process_fetch(&self) -> anyhow::Result<Mint> {
        let rpc_client = RpcClient::new(self.solana.solana_rpc_url.clone());
        let acc = rpc_client.get_account(&self.mint_pubkey).await?;

        let mint =
            Mint::unpack(&acc.data).map_err(|e| anyhow::anyhow!("Error unpacking mint: {}", e))?;

        Ok(mint)
    }
}

#[derive(Debug, Subcommand)]
pub enum MintInstructions {
    Create,
}
