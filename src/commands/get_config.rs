use super::*;
use crate::utils::config::get_config;
use crate::utils::table::Table;
use crate::utils::to_btreemap::ToBTreeMap;
use anyhow::Context;
use anyhow::Result;

/// Get the configuration either as a table or as a JSON output
#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    keys_only: bool,
}

pub async fn command(args: Args, json: bool) -> Result<()> {
    let config = get_config()?;

    if json {
        let json = serde_json::to_string_pretty(&config).context("Failed to serialize")?;
        println!("{}", json);
        return Ok(());
    };

    if args.keys_only {
        let key_map = config.keys.to_btreemap()?;
        Table::new("Fingerprint | Key ID".into(), key_map).print()?;
        return Ok(());
    };

    Table::new("Configuration".into(), config.to_btreemap()?).print()?;

    Ok(())
}
