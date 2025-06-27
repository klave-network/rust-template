use musig2::secp256k1::{ecdsa, PublicKey, SecretKey};
use musig2::{AggNonce, FirstRound, KeyAggContext, PartialSignature, PubNonce, SecNonceSpices};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::utils::{generate_rnd_bytes, secp_sign};
use crate::wallet::{KeyPairString, MusigSessionDefinition};

#[derive(Serialize, Deserialize, Debug)]
struct InputNonceCreation{
    musig_session_id: String,
    key_agg_context: String
}

#[derive(Serialize, Deserialize, Debug)]
struct InputPartialSigCreation{
    musig_session_id: String,
    key_agg_context: String,
    agg_nonce: String
}

#[derive(Serialize, Deserialize, Debug)]
struct MusigFirstRound{
    sender_id: String,

}

#[derive(Serialize, Deserialize, Debug)]
struct NonceSubmission {
    musig_session_id: String,
    public_nonce: PubNonce,
    signer_index: i8,
    message: String,
    signature: ecdsa::Signature
}

#[derive(Serialize, Deserialize, Debug)]
struct PartialSignatureSubmission {
    musig_session_id: String,
    signer_index: i8,
    partial_signature: PartialSignature
}

#[derive(Serialize, Deserialize, Debug)]
struct FinalSignatureInput {
    key_agg_context: String,
    final_signature:String,
    message:String
}

pub fn create_public_nonce(cmd: String){
    //get input
    let Ok(res) = serde_json::from_str::<InputNonceCreation>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as InputNonceCreation", cmd));
        return
    };
    //get sender id
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    //deserialize key aggregated context
    let Ok(ctx_agg) = KeyAggContext::from_hex(&res.key_agg_context) else {
        klave::notifier::send_string("ERROR: failed to parse key aggregated context");
        return
    };
    //generate nonce
    let Ok(nonce_seed) = generate_rnd_bytes() else {
        klave::notifier::send_string("ERROR: failed to generate key id");
        return
    };
    //save nonce to reproduce later on the public nonce
    let mut map_musig_seed_nonce = klave::ledger::get_table("musig_user_seed_nonce_tab").get_json::<Map<String,Value>>(&sender_id).unwrap_or(Map::new());
    map_musig_seed_nonce.insert(res.musig_session_id.clone(), Value::String(hex::encode(nonce_seed)));
    let Ok(()) = klave::ledger::get_table("musig_user_seed_nonce_tab").set_json::<Map<String, Value>>(&sender_id, &map_musig_seed_nonce) else {
        klave::notifier::send_string("ERROR: failed to save the nonce seed");
        return
    };

    //get the session
    let Ok(session) = klave::ledger::get_table("musig_sessions_def_tab").get_json::<MusigSessionDefinition>(&res.musig_session_id) else {
        klave::notifier::send_string("ERROR: failed to load session");
        return
    };
    //get the signer index
    if session.signer_indexes.is_none() {
        klave::notifier::send_string("ERROR: failed to load signer indexes");
        return
    }
    let signer_index;
    match session.signer_indexes {
        Some(value) => {
            signer_index = *value.get(&sender_id).unwrap();
        },
        None => {
            klave::notifier::send_string("ERROR: failed to load signer indexes");
            return
        }
    };
    //get the key name associated with this public key
    let map_user_session_key = klave::ledger::get_table("musig_sessions_user_key_name").get_json::<Map<String,Value>>(&sender_id)
        .unwrap_or(Map::new());
    let key_name: String;
    match map_user_session_key.get(&res.musig_session_id) {
        Some(value) => {
            key_name = value.as_str().unwrap().to_string();
        },
        None => {
            klave::notifier::send_string("ERROR: failed to get key name");
            return
        }
    }
    //retrieve list of user keys
    let user_key_ids_map: Map<String, Value> = match klave::ledger::get_table("user_key_ids_tab").get_json(&sender_id) {
        Ok(json) => json,
        Err(_) => Map::new()
    };
    if !user_key_ids_map.contains_key(&key_name){
        klave::notifier::send_string("ERROR: failed to load key from key name");
        return
    }
    //get the secret key
    let key_id;
    match user_key_ids_map.get(&key_name) {
        Some(value) => {
            key_id = value.as_str().unwrap().to_string();
        },
        None => {
            klave::notifier::send_string("ERROR: failed to get key id");
            return
        }
    }
    let Ok(key_pair) = klave::ledger::get_table("users_keys_tab").get_json::<KeyPairString>(&key_id) else {
        klave::notifier::send_string("ERROR: failed to load the key pair");
        return
    };
    let secret_key = key_pair.sec_key.parse::<SecretKey>().unwrap();
    //first round
    let first_round = FirstRound::new(
        ctx_agg.clone(),
        nonce_seed,
        signer_index.try_into().unwrap(),
        SecNonceSpices::new()
            .with_seckey(secret_key)
            .with_message(&session.msg)
    ).unwrap();

    //public nonce
    let pub_nonce = first_round.our_public_nonce();

    //signature
    let Ok(sig) = secp_sign(session.msg.clone(), secret_key) else {
        klave::notifier::send_string("ERROR: failed to sign");
        return
    };

    //final result
    let nonce_submission = NonceSubmission {
        musig_session_id : session.session_id,
        public_nonce: pub_nonce,
        signer_index: signer_index,
        message: session.msg,
        signature: sig
    };

    let _ = klave::notifier::send_json(&nonce_submission);
}

pub fn create_partial_signature(cmd: String){
    //get input
    let Ok(res) = serde_json::from_str::<InputPartialSigCreation>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as InputNonceCreation", cmd));
        return
    };
    //get sender id
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    //deserialize key aggregated context
    let Ok(ctx_agg) = KeyAggContext::from_hex(&res.key_agg_context) else {
        klave::notifier::send_string("ERROR: failed to parse key aggregated context");
        return
    };
    //deserialize aggregated nonce
    let Ok(agg_nonce) = AggNonce::from_hex(&res.agg_nonce) else {
        klave::notifier::send_string("ERROR: failed to parse aggregated nonce");
        return
    };
    //get nonce seed
    let map_musig_seed_nonce = klave::ledger::get_table("musig_user_seed_nonce_tab").get_json::<Map<String,Value>>(&sender_id).unwrap_or(Map::new());
    if !map_musig_seed_nonce.contains_key(&res.musig_session_id)
    {
        klave::notifier::send_string("ERROR: failed to retrieve the nonce seed");
        return
    }

    let nonce_seed_str: String;
    match map_musig_seed_nonce.get(&res.musig_session_id) {
        Some(value) => nonce_seed_str = value.as_str().unwrap().to_string(),
        None => {
            klave::notifier::send_string("ERROR: failed to get nonce seed");
            return
        }
    }
    let nonce_seed : [u8;32] = hex::decode(nonce_seed_str).unwrap().try_into().unwrap();
    //get the session
    let Ok(session) = klave::ledger::get_table("musig_sessions_def_tab").get_json::<MusigSessionDefinition>(&res.musig_session_id) else {
        klave::notifier::send_string("ERROR: failed to load session");
        return
    };
    //get the signer index
    if session.signer_indexes.is_none() {
        klave::notifier::send_string("ERROR: failed to load signer indexes");
        return
    }
    let signer_index: i8;
    match session.signer_indexes {
        Some(value) => {
            signer_index = *value.get(&sender_id).unwrap();
        },
        None => {
            klave::notifier::send_string("ERROR: failed to load signer indexes");
            return
        }
    };
    //get the key name associated with this public key
    let map_user_session_key = klave::ledger::get_table("musig_sessions_user_key_name").get_json::<Map<String,Value>>(&sender_id)
        .unwrap_or(Map::new());
    let key_name: String;
    match map_user_session_key.get(&res.musig_session_id) {
        Some(value) => key_name = value.as_str().unwrap().to_string(),
        None => {
            klave::notifier::send_string("ERROR: failed to get key name");
            return
        }
    }
    //retrieve list of user keys
    let user_key_ids_map: Map<String, Value> = match klave::ledger::get_table("user_key_ids_tab").get_json(&sender_id) {
        Ok(json) => json,
        Err(_) => Map::new()
    };
    if !user_key_ids_map.contains_key(&key_name){
        klave::notifier::send_string("ERROR: failed to load key from key name");
        return
    }

    //get the secret key
    let key_id:String;
    match user_key_ids_map.get(&key_name) {
        Some(value) => key_id = value.as_str().unwrap().to_string(),
        None => {
            klave::notifier::send_string("ERROR: failed to get key id");
            return
        }
    }
    let Ok(key_pair) = klave::ledger::get_table("users_keys_tab").get_json::<KeyPairString>(&key_id) else {
        klave::notifier::send_string("ERROR: failed to load the key pair");
        return
    };
    let secret_key = key_pair.sec_key.parse::<SecretKey>().unwrap();
    //replay first round
    let first_round = FirstRound::new(
        ctx_agg.clone(),
        nonce_seed,
        signer_index.try_into().unwrap(),
        SecNonceSpices::new()
            .with_seckey(secret_key)
            .with_message(&session.msg)
    ).unwrap();

    //compute partial signature
    let partial_signature: PartialSignature = first_round
    .sign_for_aggregator(secret_key, session.msg, &agg_nonce)
    .unwrap();

    //result to submit for partial signature
    let partial_sig_submission = PartialSignatureSubmission{
        musig_session_id: res.musig_session_id,
        signer_index: signer_index,
        partial_signature: partial_signature
    };

    let _ = klave::notifier::send_json::<PartialSignatureSubmission>(&partial_sig_submission);

}

pub fn verify_final_signature(cmd: String){
    //deserialize input
    let Ok(res) = serde_json::from_str::<FinalSignatureInput>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as FinalSignatureInput", cmd));
        return
    };
    //deserialize key aggregated context
    let Ok(ctx_agg) = KeyAggContext::from_hex(&res.key_agg_context) else {
        klave::notifier::send_string("ERROR: failed to parse key aggregated context");
        return
    };
    //Convert signature to [u8;64]
    let sig: [u8;64] = hex::decode(res.final_signature).unwrap().try_into().unwrap();

    //verify signature
    let agg_pub_key: PublicKey = ctx_agg.aggregated_pubkey();
    match musig2::verify_single(agg_pub_key, sig, res.message){
        Ok(()) => {
            klave::notifier::send_string("Signature verified");
            return
        },
        Err(e) => {
            klave::notifier::send_string(&format!("Signature not verified with error: '{}'",e));
            return
        }
    }
}