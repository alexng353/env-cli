use super::*;
use crate::utils::{
    config::get_config,
    e::{decrypt, get_vault_location},
};
use anyhow::{Context, Result};
use hex::ToHex;
use pgp::{composed, types::KeyTrait, Deserializable, Message, SignedSecretKey};
use std::{io::Cursor, vec};

/// Decrypt a string using GPG
#[derive(Parser)]
pub struct Args {}

pub async fn command(args: Args, _json: bool) -> Result<()> {
    let config = get_config().context("Failed to get config")?;
    let msg = r#"-----BEGIN PGP MESSAGE-----

wcBMA0kZBUywA0h0AQgAqV8T3LtWijYa4btDD4kzVpEYuvmhRaG4BRElzf7Mr2tK
9xDK0KVnW3ncxhiwf4UmI3jBqdkGAqGePKhSSHsGDz6zlGnOLI0q2AdicVkH2XBG
n5AjOdsF6xRq6UO+8hCyH7vGaU6ZZd9H4ypd1RiBCbF/1Dwh5w7MhZj3c1/9Pt1u
z4yRcts/JZQlPhfcivumDFBEeQq9Y90Gn3/+qpeuTZn9WlFBkuW+IqUaUnBYLa0t
0lpVqpby/I8DX1mYFd24mLJsXT9VswS0BbhU9XbRWcBIKFBRK3UgZ/5M74OXILY8
xA8mQiyWGu9b6ddZwybtPrUl1QZVbHGjs/7dXmEXrNI6Aaiz+XlpUUpV+c3rA9vl
S3gZvI6pms0dKbrfU40oVXyFgWuoLMcP92LdpLOWixZs2O/4oonDILElVA==
=uE8A

-----END PGP MESSAGE-----"#;

    let msg = r#"-----BEGIN PGP MESSAGE-----

wcBMA0kZBUywA0h0AQgAhT41iqKXwA1AcfcTtsUtpqGNH7iSolNtUTk2HqetjTsU
7aSQlCkaCEX4k4vB3gGkwHbP1VgkuGFEx17rdTn35FfxJglzT8u2LRSV2DOL3Y6U
n62MCtLpc0e4mGg6gxV63eNfFf9k4aKoqJu9qgZuZbLOzNWk2Ed8sxH46318O+mA
gTIhHzKFNUezC8+MFokbRic6v97kCbhd/UQrvIZjZ0/+vuOXOiiMzsoGoy5QQoLf
6gJmjKCW42zBvcPVv/l7L24/LLdRVwggBWQsBqt6WtCFN+2dqxulH4X+bMOjgUO3
CvuTyM2pgyU0rovuofMnNgy0cxtlweHWar3POIOvJtJBAVikG0kKx6nSIbqdCUQT
khmVI7CGUL6s4wFRQnOp7YMKy/gf1mO0PtRtGkcuPEsUwWElFrcig5rBBl+VtBcM
wD8=
=6pJ1
-----END PGP MESSAGE-----"#;

    let buf = Cursor::new(msg);

    let (msg, _) = composed::message::Message::from_armor_single(buf)
        .context("Failed to convert &str to armored message")?;

    let recipients = msg
        .get_recipients()
        .iter()
        .map(|e| e.encode_hex_upper())
        .collect::<Vec<String>>();

    let keys = config.keys.clone();

    let mut available_keys: Vec<String> = vec![];

    for key in keys {
        let it_fits = recipients.iter().any(|r| key.contains(r));
        if it_fits {
            available_keys.push(key);
        }
    }

    if available_keys.len() == 0 {
        eprintln!("No keys available to decrypt this message");
        return Ok(());
    }

    let primary_key = config.primary_key.clone();

    let (key, fingerprint) = if available_keys.iter().any(|k| k.contains(&primary_key)) {
        println!("Using primary key");
        get_key(primary_key)
    } else {
        println!("Using first available key: {}", available_keys[0].clone());
        get_key(available_keys[0].clone())
    };

    println!("Using key: {}", fingerprint);

    println!("Decrypting...\n\n");

    let decrypted = decrypt(
        &msg.to_armored_string(None).unwrap(),
        &key,
        String::from("asdf"),
    )?;

    println!("{}", decrypted);

    Ok(())
}

fn get_key(fingerprint: String) -> (SignedSecretKey, String) {
    let key_dir = get_vault_location().unwrap().join(fingerprint.clone());
    let priv_key = std::fs::read_to_string(key_dir.join("private.key")).unwrap();
    let (seckey, _) = SignedSecretKey::from_string(priv_key.as_str()).unwrap();

    (seckey, fingerprint)
}
