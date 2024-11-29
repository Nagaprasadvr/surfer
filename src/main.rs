use clap::Parser;
use cli::Cli;

mod cli;
mod mint;
mod token_account;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.validate_args()?;

    match cli.command {
        cli::Commands::Mint(m_cmd) => {
            m_cmd.process().await?;
        }
        cli::Commands::TokenAccount(ta_cmd) => {
            ta_cmd.process().await?;
        }
    }

    Ok(())
}
