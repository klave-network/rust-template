use musig2::{
    self,
    secp256k1::{
        ecdsa::{self},
        Message, PublicKey, Secp256k1,
    },
    verify_partial, AggNonce, KeyAggContext, PartialSignature, PubNonce,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
struct InputMusigInitiation {
    public_keys: Vec<String>,
    initiating_key: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MusigSession {
    id: String,
    sender_id: String,
    key_agg_context: KeyAggContext,
    public_keys: Vec<String>,
    initiating_key: String,
    step: i8,
    message: String,
    pub_nonces: Option<Map<String, Value>>,
    agg_nonce: Option<AggNonce>,
    partial_signatures: Option<Map<String, Value>>,
    final_signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NonceSubmission {
    musig_session_id: String,
    public_nonce: PubNonce,
    signer_index: i8,
    message: String,
    signature: ecdsa::Signature,
}

#[derive(Serialize, Deserialize, Debug)]
struct PartialSignatureSubmission {
    musig_session_id: String,
    signer_index: i8,
    partial_signature: PartialSignature,
}

#[derive(Serialize, Deserialize, Debug)]
struct InputTest {
    item: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MusigUserSessions {
    musig_sessions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MusigSessionId {
    musig_session_id: String,
}

pub fn create_musig_context(
    public_keys: Vec<String>,
) -> Result<KeyAggContext, Box<dyn std::error::Error>> {
    if !public_keys.windows(2).all(|w| w[0] <= w[1]) {
        return Err("Public keys must be sorted".into());
    }

    let pub_keys: Vec<PublicKey> = public_keys
        .iter()
        .map(|pk| pk.parse::<PublicKey>())
        .collect::<Result<Vec<PublicKey>, _>>()?;

    let ctx: KeyAggContext = musig2::KeyAggContext::new(pub_keys)?;
    Ok(ctx)
}

pub fn initiate_musig_context(cmd: String) {
    let Ok(res) = serde_json::from_str::<InputMusigInitiation>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{cmd}' as InputMusigInitiation"));
        return;
    };
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("failed to get the sender id");
        return;
    };

    //creation of musig public keys aggregation context
    let ctx: KeyAggContext = match create_musig_context(res.public_keys.clone()) {
        Err(e) => {
            klave::notifier::send_string(&format!("failed to create musig context: '{e}'"));
            return;
        }
        Ok(context) => context,
    };

    let Ok(musig_id) = klave::crypto::random::get_random_bytes(32).map(hex::encode) else {
        klave::notifier::send_string("failed to generate musig id");
        return;
    };
    let musig_session = MusigSession {
        id: musig_id.clone(),
        sender_id: sender_id.clone(),
        key_agg_context: ctx,
        public_keys: res.public_keys.clone(),
        initiating_key: res.initiating_key,
        step: 0,
        message: res.message.clone(),
        pub_nonces: None,
        agg_nonce: None,
        partial_signatures: None,
        final_signature: None,
    };

    //save musig session to ledger
    if let Err(e) =
        klave::ledger::get_table("musig_session_tab").set_json(&musig_id, &musig_session)
    {
        klave::notifier::send_string(&format!("failed to write to ledger: '{e}'"));
        // sdk::cancel_transaction();
        return;
    }

    //associate musig session id with the sender id
    let mut musig_user_sessions: Vec<String> = vec![];
    if let Ok(vec) = klave::ledger::get_table("musig_user_agg_sessions_tab").get_json(&sender_id) {
        musig_user_sessions = vec;
    }
    musig_user_sessions.push(musig_id.clone());
    if let Err(e) = klave::ledger::get_table("musig_user_agg_sessions_tab")
        .set_json(&sender_id, &musig_user_sessions)
    {
        klave::notifier::send_string(&format!("failed to write to ledger: '{e}'"));
        // sdk::cancel_transaction();
        return;
    }

    klave::notifier::send_string(musig_id.as_str());
}

pub fn load_musig_agg_session_ids(_cmd: String) {
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("failed to get the sender id");
        return;
    };
    let mut musig_user_sessions: Vec<String> = vec![];
    if let Ok(vec) = klave::ledger::get_table("musig_user_agg_sessions_tab").get_json(&sender_id) {
        musig_user_sessions = vec;
    }
    let _ = klave::notifier::send_json::<Vec<String>>(&musig_user_sessions);
}

pub fn load_musig_agg_session(cmd: String) {
    let Ok(musig_session_id) = serde_json::from_str::<MusigSessionId>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{cmd}' as InputMusigInitiation"));
        return;
    };
    let Ok(musig_session) =
        klave::ledger::get_table("musig_session_tab").get_json(&musig_session_id.musig_session_id)
    else {
        klave::notifier::send_string("failed to get musig session");
        return;
    };
    let _ = klave::notifier::send_json::<MusigSession>(&musig_session);
}

pub fn submit_pub_nonce(cmd: String) {
    let Ok(nonce_submission) = serde_json::from_str::<NonceSubmission>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{cmd}' as NonceSubmission"));
        return;
    };

    let Ok(mut musig_session) = klave::ledger::get_table("musig_session_tab")
        .get_json::<MusigSession>(&nonce_submission.musig_session_id)
    else {
        klave::notifier::send_string("failed to get musig session");
        return;
    };

    if musig_session.step >= 1 {
        klave::notifier::send_string("nonce submissions already completed");
        return;
    }
    let Ok(pub_key) =
        musig_session.public_keys[nonce_submission.signer_index as usize].parse::<PublicKey>()
    else {
        klave::notifier::send_string("failed to parse public key");
        return;
    };

    let secp = Secp256k1::new();
    let digest = Sha256::digest(nonce_submission.message.as_bytes());
    let message = Message::from_digest(digest.into());
    //submission can only happen once so a replay attack is not possible
    let Ok(()) = pub_key.verify(&secp, &message, &nonce_submission.signature) else {
        klave::notifier::send_string("failed to verify signature");
        return;
    };

    //nonce can be submitted only once
    let mut pub_nonces = musig_session.pub_nonces.unwrap_or(Map::new());
    if pub_nonces
        .contains_key(musig_session.public_keys[nonce_submission.signer_index as usize].as_str())
    {
        klave::notifier::send_string("nonce already submitted");
        return;
    }

    //nonce submission
    pub_nonces.insert(
        musig_session.public_keys[nonce_submission.signer_index as usize].clone(),
        serde_json::to_value(nonce_submission.public_nonce).unwrap(),
    );
    let nonce_count = pub_nonces.len();

    //all nonces have been submitted and it's time to aggregate
    if nonce_count == musig_session.public_keys.len() {
        musig_session.step = 1;
        let aggregated_nonce = AggNonce::sum(
            pub_nonces
                .values()
                .map(|v| serde_json::from_value::<PubNonce>(v.clone()).unwrap())
                .collect::<Vec<PubNonce>>(),
        );
        musig_session.pub_nonces = Some(pub_nonces);
        musig_session.agg_nonce = Some(aggregated_nonce);
        //save musig session to ledger
        match klave::ledger::get_table("musig_session_tab")
            .set_json(&nonce_submission.musig_session_id, &musig_session)
        {
            Err(e) => {
                klave::notifier::send_string(&format!("failed to write to ledger: '{e}'"));
                return;
            }
            Ok(()) => {
                let _ = klave::notifier::send_json::<MusigSession>(&musig_session);
                return;
            }
        }
    }
    //only a subset of nonces have been submitted
    musig_session.pub_nonces = Some(pub_nonces);
    //save musig session to ledger
    match klave::ledger::get_table("musig_session_tab")
        .set_json(&nonce_submission.musig_session_id, &musig_session)
    {
        Err(e) => {
            klave::notifier::send_string(&format!("failed to write to ledger: '{e}'"));
        }
        Ok(()) => {
            let _ = klave::notifier::send_json::<MusigSession>(&musig_session);
        }
    }
}

pub fn submit_partial_signature(cmd: String) {
    let Ok(partial_signature_submission) = serde_json::from_str::<PartialSignatureSubmission>(&cmd)
    else {
        klave::notifier::send_string(&format!(
            "failed to parse '{cmd}' as PartialSignatureSubmission"
        ));
        return;
    };
    let Ok(mut musig_session) = klave::ledger::get_table("musig_session_tab")
        .get_json::<MusigSession>(&partial_signature_submission.musig_session_id)
    else {
        klave::notifier::send_string("failed to get musig session");
        return;
    };
    if musig_session.step != 1 {
        klave::notifier::send_string("partial signature submission not allowed at this step");
        return;
    }

    let signer_index = partial_signature_submission.signer_index;
    let pub_key: PublicKey = musig_session.public_keys[signer_index as usize]
        .parse::<PublicKey>()
        .unwrap();

    let agg_nonce: AggNonce = match musig_session.agg_nonce {
        Some(res) => res,
        None => {
            klave::notifier::send_string("failed to get aggregated nonce");
            return;
        }
    };

    let pub_nonce: PubNonce = match musig_session.pub_nonces.clone() {
        Some(res) => {
            serde_json::from_value::<PubNonce>(res.get(&pub_key.to_string()).unwrap().clone())
                .unwrap()
        }
        None => {
            klave::notifier::send_string("failed to get public nonce");
            return;
        }
    };

    let Ok(()) = verify_partial(
        &musig_session.key_agg_context,
        partial_signature_submission.partial_signature,
        &agg_nonce.clone(),
        pub_key,
        &pub_nonce,
        &musig_session.message,
    ) else {
        klave::notifier::send_string("failed to verify partial signature");
        return;
    };

    //partial signature can be submitted only once
    let mut partial_signatures = musig_session.partial_signatures.unwrap_or(Map::new());
    if partial_signatures.contains_key(&pub_key.to_string()) {
        klave::notifier::send_string("partial signature already submitted");
        return;
    }

    //partial signature submission
    partial_signatures.insert(
        pub_key.to_string(),
        serde_json::to_value(partial_signature_submission.partial_signature).unwrap(),
    );

    //check the number of partial signatures submitted
    let sig_count = partial_signatures.len();
    musig_session.partial_signatures = Some(partial_signatures.clone());
    if sig_count == musig_session.public_keys.len() {
        musig_session.step = 2;
        let partial_sigs: Vec<PartialSignature> = partial_signatures
            .values()
            .map(|v| serde_json::from_value::<PartialSignature>(v.clone()).unwrap())
            .collect();
        let final_sig: [u8; 64] = musig2::aggregate_partial_signatures(
            &musig_session.key_agg_context,
            &agg_nonce,
            partial_sigs,
            musig_session.message.clone(),
        )
        .unwrap();
        musig_session.final_signature = Some(hex::encode(final_sig));
    }

    let agg_nonce2 = agg_nonce.clone();
    musig_session.agg_nonce = Some(agg_nonce2);

    //save musig session to ledger
    match klave::ledger::get_table("musig_session_tab").set_json(
        &partial_signature_submission.musig_session_id,
        &musig_session,
    ) {
        Err(e) => {
            klave::notifier::send_string(&format!("failed to write to ledger: '{e}'"));
        }
        Ok(()) => {
            let _ = klave::notifier::send_json::<MusigSession>(&musig_session);
        }
    }
}

pub fn get_final_signature(cmd: String) {
    let Ok(musig_session_id) = serde_json::from_str::<MusigSessionId>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{cmd}' as MusigSessionId"));
        return;
    };
    let Ok(musig_session) = klave::ledger::get_table("musig_session_tab")
        .get_json::<MusigSession>(&musig_session_id.musig_session_id)
    else {
        klave::notifier::send_string("failed to get musig session");
        return;
    };

    let final_sig: String = match musig_session.final_signature.clone() {
        Some(res) => res,
        None => {
            klave::notifier::send_string("failed to get final signature");
            return;
        }
    };

    klave::notifier::send_string(&final_sig);
}
mod tests {
    use crate::musig_agg::{create_musig_context, InputMusigInitiation};
    use musig2::{secp256k1::PublicKey, KeyAggContext};

    #[test]
    fn test_sorting_change_pubkey() {
        let key1: String =
            "026e14224899cf9c780fef5dd200f92a28cc67f71c0af6fe30b5657ffc943f08f4".to_string();
        let key2: String =
            "02f3b071c064f115ca762ed88c3efd1927ea657c7949698b77255ea25751331f0b".to_string();
        let key3: String =
            "03204ea8bc3425b2cbc9cb20617f67dc6b202467591d0b26d059e370b71ee392eb".to_string();
        let pubkeys: Vec<String> = vec![key1.clone(), key2, key3];
        let message = "hello world".to_string();

        let json_initiation: InputMusigInitiation = InputMusigInitiation {
            public_keys: pubkeys,
            initiating_key: key1.clone(),
            message,
        };

        let ctx = create_musig_context(json_initiation.public_keys).unwrap();

        let json_ctx = serde_json::to_string(&ctx).unwrap();
        let ctx2 = serde_json::from_str::<KeyAggContext>(&json_ctx).unwrap();

        // This is the key which the group has control over.
        let aggregated_pubkey: PublicKey = ctx.aggregated_pubkey();
        assert_eq!(
            aggregated_pubkey.to_string(),
            "02e272de44ea720667aba55341a1a761c0fc8fbe294aa31dbaf1cff80f1c2fd940".to_string()
        );

        let aggregated_pubkey2: PublicKey = ctx2.aggregated_pubkey();
        assert_eq!(
            aggregated_pubkey.to_string(),
            aggregated_pubkey2.to_string()
        );
    }
}
