use clap::Subcommand;

use crate::cli::FetchAccount;

#[derive(Debug, Subcommand)]
pub enum TokenAccountCommands {
    Fetch(FetchAccount),
    #[clap(subcommand)]
    Ix(TokenAccountInstructions),
}

impl TokenAccountCommands {
    pub fn process(&self) {
        match self {
            TokenAccountCommands::Fetch(fetch) => {
                println!("Fetch command:{:?}", fetch);
            }
            TokenAccountCommands::Ix(ix) => {
                println!("Ix command:{:?}", ix);
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TokenAccountInstructions {
    Create,
}
