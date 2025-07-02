use hex::encode;
use klave::crypto::subtle::{self, CryptoKey, EncryptAlgorithm, KeyDerivationAlgorithm, HkdfDerivParams, AesGcmParams, AesKeyGenParams, DerivedKeyAlgorithm, derive_key, encrypt, export_key};
use serde_json::Value;
use crate::utils::get_serde_value_into_bytes;

// AES-GCM constants
pub const AES_GCM_IV_SIZE: usize = 12;      // 12 bytes (96 bits) - optimal for AES-GCM


pub fn generate_ecc_crypto_key() -> Result<CryptoKey, Box<dyn std::error::Error>> {
    let ec_params = subtle::EcKeyGenParams { named_curve: "P-256".to_string() };
    let gen_algorithm = subtle::KeyGenAlgorithm::Ecc(ec_params);

    let private_key = match subtle::generate_key(&gen_algorithm, false, &["sign", "derive_key"]) {
        Ok(result) => result,
        Err(err) => {
            klave::notifier::send_string(&err.to_string());
            return Err(err.into());
        }
    };
    Ok(private_key)
}

pub fn compute_sha256_hex_string(data: &[u8]) -> String {
    // Using Klave's crypto utilities
    match klave::crypto::sha::digest("SHA2-256", data) {
        Ok(hash) => hex::encode(hash),
        Err(e) => {
            klave::notifier::send_string(&format!("SHA2-256 computation failed: {}", e));
            String::new()
        }
    }
}

pub fn derive_aes_gcm_key(master_key: &CryptoKey, table: String, column_name: String) -> Result<CryptoKey, Box<dyn std::error::Error>> {
    // Use HKDF to derive a key from the master key and the column name
    let hkdf_derivation_params = HkdfDerivParams {
        hash: "SHA-256".to_string(),
        salt: format!("klave-salt-encryption-'{}'", table).into_bytes(),
        // Use the value as info to ensure uniqueness per column and row
        info: format!("klave-info-encryption-'{}'", column_name).into_bytes(),
    };
    let derivation_algorithm = KeyDerivationAlgorithm::Hkdf(hkdf_derivation_params);
    let aes_key_gen_params = AesKeyGenParams {
        length: 256, // AES-256
    };
    let derived_key_algorithm = DerivedKeyAlgorithm::Aes(aes_key_gen_params);
    let extractable = false;
    let usages = ["encrypt", "decrypt"];
    let aes_gcm_key = match derive_key(&derivation_algorithm, &master_key, &derived_key_algorithm, extractable, &usages) {
        Ok(key) => key,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to derive key: {}", err));
            return Err(err.into());
        }
    };
    Ok(aes_gcm_key)
}

pub fn derive_iv(master_key: &CryptoKey, column_name: String, value: Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let value_in_bytes = match get_serde_value_into_bytes(&value) {
        Ok(bytes) => bytes,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to convert value to bytes: {}", err));
            return Err(err.into());
        }
    };
    let salt = match klave::crypto::sha::digest("SHA-256", &value_in_bytes)
    {
        Ok(s) => s,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to compute salt: {}", err));
            return Err(err.into());
        }
    };
    let hkdf_deriv_params_iv = HkdfDerivParams {
        hash: "SHA-256".to_string(),
        info: format!("klave-iv-'{}", column_name).into_bytes(),
        salt: salt,
    };
    let deriv_algo_iv = KeyDerivationAlgorithm::Hkdf(hkdf_deriv_params_iv);
    let aes_key_gen_params = AesKeyGenParams {
        length: 256, // AES-256
    };
    let derived_key_algorithm = DerivedKeyAlgorithm::Aes(aes_key_gen_params);
    let usages = ["encrypt", "decrypt"];
    let extractable = true;
    let iv_key = match derive_key(&deriv_algo_iv, &master_key, &derived_key_algorithm, extractable, &usages) {
        Ok(key) => key,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to derive key: {}", err));
            return Err(err.into());
        }
    };
    let iv = match export_key("raw", &iv_key)
    {
        Ok(mut iv) => {iv.truncate(AES_GCM_IV_SIZE as usize); iv},
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to export key: {}", err));
            return Err(err.into());
        }
    };
    Ok(iv)
}

pub fn encrypt_value(master_key: &CryptoKey, table_name: String, column_name: String, value:Value) -> Result<String, Box<dyn std::error::Error>> {
    // Convert serde Value in bytes
    let value_in_bytes = match get_serde_value_into_bytes(&value) {
        Ok(bytes) => bytes,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to convert value to bytes: {}", err));
            return Err(err);
        }
    };

    // Derive AES-GCM key for the column
    let aes_gcm_key = match derive_aes_gcm_key(&master_key,table_name.clone(), column_name.clone()) {
        Ok(key) => key,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to derive AES-GCM key: {}", err));
            return Err(err);
        }
    };
    // Compute the iv deterministically from the point of view of the value to encrypt.
    // I derive a key from the master key and the value to encrypt, export it as raw bytes, and use the first 12 bytes as the iv.
    let iv = match derive_iv(&master_key, column_name.clone(), value.clone())
    {
        Ok(res) => res,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to derive AES-GCM key: {}", err));
            return Err(err);
        }
    };
    let iv_12 = iv[0..12].to_vec();

    // Encrypt the value with the derived AES-GCM key
    let aes_gcm_params = AesGcmParams {
        iv: iv_12.clone(),
        additional_data: vec![], // No additional data
        tag_length: 128, // 128 bits
    };
    let encrypt_algo = EncryptAlgorithm::AesGcm(aes_gcm_params);
    let mut encrypted_value = match encrypt(&encrypt_algo, &aes_gcm_key, &value_in_bytes) {
        Ok(encrypted) => encrypted,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to encrypt value: {}", err));
            return Err(err);
        }
    };
    // Concatenate iv and encrypted value
    let mut iv_and_encrypted = iv;
    iv_and_encrypted.append(&mut encrypted_value);
    // Encode the iv and encrypted value as a hex string
    let encoded_iv_value = encode(&iv_and_encrypted);

    Ok(encoded_iv_value)
}