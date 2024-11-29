use std::time::Duration;

use indicatif::ProgressBar;

pub fn get_spinner() -> anyhow::Result<ProgressBar> {
    let mut spinner = ProgressBar::new_spinner();
    spinner.set_tab_width(16);
    spinner.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg:.bold}")
            .map_err(|e| anyhow::anyhow!("Error creating spinner: {}", e))?,
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Fetching mint data...");

    Ok(spinner)
}
