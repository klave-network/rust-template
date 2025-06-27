use std::str::FromStr;

use klave::crypto::{self};
use musig2::secp256k1::{ecdsa::Signature, Message, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

pub fn generate_secp256k1_secret_key() -> Result<String, Box<dyn std::error::Error>> {
    let array_rng : [u8;32] = crypto::random::get_random_bytes(32)?.as_slice().try_into()?;
    let seckey = format!("{}", SecretKey::from_byte_array(&array_rng)?.display_secret());
    Ok(seckey)
}

pub fn generate_public_key_from_secret_key(seckey: String) -> Result<String, Box<dyn std::error::Error>> {
    let seckey = SecretKey::from_str(seckey.as_str())?;
    let secp = Secp256k1::new();
    let pubkey = seckey.public_key(&secp).to_string();
    Ok(pubkey)
}

pub fn generate_rng_id() -> Result<String, Box<dyn std::error::Error>> {
    let rng_id : String = hex::encode(crypto::random::get_random_bytes(32)?);
    Ok(rng_id)
}

pub fn generate_rnd_bytes() -> Result<[u8;32], Box<dyn std::error::Error>> {
    let rnd_bytes : [u8;32] = crypto::random::get_random_bytes(32)?.as_slice().try_into()?;
    Ok(rnd_bytes)
}

pub fn secp_sign(message: String, secret_key: SecretKey) -> Result<Signature, Box<dyn std::error::Error>>{
    let digest = Sha256::digest(&message.as_bytes());
    let msg = Message::from_digest(digest.into());
    let secp = Secp256k1::new();
    let sig = secp.sign_ecdsa(&msg, &secret_key);
    Ok(sig)
}