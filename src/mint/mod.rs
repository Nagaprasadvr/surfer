pub mod account;
pub mod ixs;
pub mod metadata;

use std::time::Duration;

pub use account::*;
use clap::Subcommand;
use inquire::Select;
pub use ixs::*;
pub use metadata::*;

use crate::utils;

#[derive(Debug, Subcommand)]
pub enum MintCommands {
    Fetch(FetchMint),
    Ix,
}

impl MintCommands {
    pub async fn process(&self) -> anyhow::Result<()> {
        match self {
            MintCommands::Fetch(f) => {
                let spinner = utils::get_spinner("Fetching mint data...")?;
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
                let _ix = MintInstructions::from_select_str(
                    Select::new(
                        "Select an instruction to execute",
                        MintInstructions::to_select_vec(),
                    )
                    .prompt()?,
                )?;
            }
        }

        Ok(())
    }
}
