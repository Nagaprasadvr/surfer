use std::time::Duration;

use clap::Subcommand;

use crate::{
    cli::{LocalWallet, SolanaRpcArgs, TokenProgram},
    mint::FetchMint,
    utils::{self, get_pubkey_from_prompt},
};

#[derive(Debug, Clone, Subcommand)]
pub enum TokenAccountInstructions {
    Create,
    Transfer,
}

impl TokenAccountInstructions {
    pub fn to_select_vec() -> Vec<&'static str> {
        vec!["Create", "Transfer"]
    }

    pub fn from_select_str(select_str: &str) -> anyhow::Result<Self> {
        match select_str {
            "Create" => Ok(Self::Create),
            "Transfer" => Ok(Self::Transfer),

            _ => Err(anyhow::anyhow!("Invalid mint instruction: {}", select_str)),
        }
    }

    pub async fn process_ix(&self, rpc: &SolanaRpcArgs) -> anyhow::Result<()> {
        let local_wallet = LocalWallet::fetch()?;
        let ix = match self {
            TokenAccountInstructions::Create => {
                println!("Create token account");

                let mint_pubkey = get_pubkey_from_prompt("Mint account pubkey")?;

                spl_associated_token_account::instruction::create_associated_token_account(
                    &local_wallet.pubkey(),
                    &local_wallet.pubkey(),
                    &mint_pubkey,
                    &spl_token_2022::ID,
                )
            }
            TokenAccountInstructions::Transfer => {
                println!("Transfer token account");

                let mint_pubkey = get_pubkey_from_prompt("Mint account pubkey")?;

                let source_pubkey = get_pubkey_from_prompt("Source account pubkey")?;
                let dest_pubkey = get_pubkey_from_prompt("Destination account pubkey")?;

                let mut amount = inquire::Text::new("Amount to transfer without decimals")
                    .prompt()
                    .expect("Failed to get amount to transfer")
                    .parse::<u64>()?;

                if source_pubkey.eq(&dest_pubkey) {
                    return Err(anyhow::anyhow!(
                        "Source and destination accounts are the same"
                    ));
                }

                let spinner = utils::get_spinner("Fetching mint data...")?;
                spinner.enable_steady_tick(Duration::from_millis(100));
                let mint_acc = FetchMint {
                    mint_pubkey,
                    solana: rpc.clone(),
                }
                .process_fetch_and_parse()
                .await?;
                spinner.finish_and_clear();

                amount = amount
                    .checked_mul(10u64.pow(mint_acc.base.get_decimals() as u32))
                    .ok_or_else(|| anyhow::anyhow!("Failed to calculate amount"))?;

                spl_token_2022::instruction::transfer_checked(
                    &TokenProgram::Token2022.into(),
                    &source_pubkey,
                    &mint_pubkey,
                    &dest_pubkey,
                    &local_wallet.pubkey(),
                    &[&local_wallet.pubkey()],
                    amount,
                    mint_acc.base.get_decimals(),
                )?
            }
        };

        let spinner = utils::get_spinner("Sending tx...")?;
        spinner.enable_steady_tick(Duration::from_millis(100));
        local_wallet
            .sign_and_send_ixs(vec![ix], &rpc.solana_rpc_url)
            .await?;
        spinner.finish_and_clear();
        Ok(())
    }
}
