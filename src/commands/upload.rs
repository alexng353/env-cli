use super::*;
use crate::{
    sdk::SDK,
    utils::{config::get_specific_config, prompt::prompt_text},
};

#[derive(Parser)]
pub struct Args {
    /// Key to sign with
    #[clap(short, long)]
    key: String,

    /// Username to add to project
    #[clap(short, long)]
    username: Option<String>,
}

pub async fn command(args: Args, _json: bool) -> Result<()> {
    let global = false;
    let mut config = get_specific_config(global)?;

    let key = config.get_key(&args.key)?;

    let username = match args.username {
        Some(u) => u,
        None => prompt_text("Username: ")?,
    };

    let id = SDK::new_user(&username, &key.public_key()?).await?;

    for k in config.keys.iter_mut() {
        if k.fingerprint == key.fingerprint {
            k.uuid = Some(id.clone());
        }
    }

    config.write(global)?;

    Ok(())
}