pub mod account;
pub mod ixs;

use std::str::FromStr;

pub use account::*;
use clap::Subcommand;
use inquire::Select;
pub use ixs::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_token::solana_program::pubkey::Pubkey;

use crate::{
    cli::SolanaRpcArgs,
    mint::{MintWithExtensions, MintWithPubkey, PrettyMint, TokenMetadata},
};

#[derive(Debug, Subcommand)]
pub enum TokenAccountCommands {
    Fetch(FetchTokenAccount),
    Ix(SolanaRpcArgs),
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

                let token_metadata = TokenMetadata::fetch_and_parse(
                    Pubkey::from_str(&token_account.mint)?,
                    &RpcClient::new(f.solana.solana_rpc_url.clone()),
                )
                .await;

                let mint_acc_data =
                    MintWithExtensions::try_parse_mint_with_extensions(mint, token_metadata)?;

                let mint: PrettyMint = MintWithPubkey {
                    mint_data: mint_acc_data,
                    pubkey: token_account.mint.to_string(),
                }
                .into();

                mint.print();
                token_account.print();
            }
            TokenAccountCommands::Ix(rpc) => {
                let ix = TokenAccountInstructions::from_select_str(
                    Select::new(
                        "Select an instruction to execute",
                        TokenAccountInstructions::to_select_vec(),
                    )
                    .prompt()?,
                )?;

                ix.process_ix(rpc).await?;
            }
        }

        Ok(())
    }
}
