use clap::Subcommand;

use crate::cli::FetchAccount;

#[derive(Debug, Subcommand)]
pub enum MintCommands {
    Fetch(FetchAccount),
    #[clap(subcommand)]
    Ix(MintInstructions),
}

impl MintCommands {
    pub fn process(&self) {
        match self {
            MintCommands::Fetch(fetch) => {
                println!("Fetch command:{:?}", fetch);
            }
            MintCommands::Ix(ix) => {
                println!("Ix command:{:?}", ix);
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum MintInstructions {
    Create,
}
