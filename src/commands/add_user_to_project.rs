use crate::{
    sdk::{get_api_url, SDK},
    utils::{
        auth::get_token, config::get_local_or_global_config, partialkey::PartialVariable,
        rpgp::encrypt_multi,
    },
};
use reqwest::header;
use serde_json::json;

use super::*;
use anyhow::Ok;

/// Add a user to a project
#[derive(Parser)]
pub struct Args {
    /// Key to sign with
    #[clap(short, long)]
    key: String,

    /// Project ID to add user to
    #[clap(short, long)]
    project_id: String,

    /// User ID to add to project
    #[clap(short, long)]
    user_id: String,
}

pub async fn command(args: Args, _json: bool) -> Result<()> {
    let config = get_local_or_global_config()?;

    let key = config.get_key(&args.key)?;
    let uuid = key.uuid.clone().unwrap();

    let (_, user_public_key_to_add) = SDK::get_user(&key.fingerprint, &uuid, &args.user_id).await?;

    let project_info = SDK::get_project_info(&args.project_id, &key.fingerprint, &uuid).await?;

    let (kvpairs, partials) = SDK::get_variables(&args.project_id, &key.fingerprint, &uuid).await?;

    let mut recipients = project_info
        .users
        .iter()
        .map(|e| e.public_key.clone())
        .collect::<Vec<String>>();

    recipients.push(user_public_key_to_add);

    let recipients = recipients.iter().map(|r| r.as_str()).collect::<Vec<&str>>();

    let messages = kvpairs
        .iter()
        .map(|k| encrypt_multi(&k.to_json().unwrap(), recipients.clone()).unwrap())
        .collect::<Vec<String>>();

    let partials = partials
        .iter()
        .zip(messages.iter())
        .map(|(p, m)| PartialVariable {
            id: p.id.clone(),
            value: m.clone(),
            project_id: p.project_id.clone(),
        })
        .collect::<Vec<PartialVariable>>();

    let body = json!({
        "variables": partials,
    });

    let client = reqwest::Client::new();
    let auth_token = get_token(&key.fingerprint, &uuid).await?;

    let res = client
        .post(&format!("{}/variables/update-many", get_api_url()?))
        .header(header::AUTHORIZATION, format!("Bearer {}", auth_token))
        .json(&body)
        .send()
        .await?
        .json::<Vec<String>>()
        .await?;

    println!("Updated {} variables", res.len());
    println!("IDs: {:?}", res);

    SDK::add_user_to_project(&key.fingerprint, &uuid, &args.user_id, &args.project_id).await?;

    Ok(())
}
