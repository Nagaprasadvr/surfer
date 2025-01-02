use std::time::Duration;

use indicatif::ProgressBar;
use spl_pod::solana_pubkey::Pubkey;

pub fn get_spinner(msg: &str) -> anyhow::Result<ProgressBar> {
    let mut spinner = ProgressBar::new_spinner();
    spinner.set_tab_width(16);
    spinner.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg:.bold}")
            .map_err(|e| anyhow::anyhow!("Error creating spinner: {}", e))?,
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(msg.to_string());

    Ok(spinner)
}

pub fn get_pubkey_from_prompt(msg: &str) -> anyhow::Result<Pubkey> {
    Ok(inquire::Text::new(msg)
        .prompt()
        .expect("Failed to get pubkey")
        .parse::<Pubkey>()?)
}
