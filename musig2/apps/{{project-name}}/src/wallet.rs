use std::collections::HashMap;

use klave::crypto;
use musig2::secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
//use klave::crypto::{self, subtle::save_key};
use serde_json::{Map, Value};

use crate::utils::{generate_public_key_from_secret_key, generate_rng_id, generate_secp256k1_secret_key};

#[derive(Serialize, Deserialize, Debug)]
struct InputRegistration {
    user_name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct InputKeyCreation {
    key_name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct InputMusigSessionInvitation {
    session_name: String,
    participants: Vec<String>,
    message: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MusigSessionDefinition {
    pub session_name: String,
    pub session_id: String,
    pub users_id: Vec<String>,
    pub owner_id: String,
    pub public_keys: Map<String,Value>,
    pub msg: String,
    pub signer_indexes: Option<HashMap<String,i8>>,
    pub musig_aggregation_session: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct InputMusigSessionUserPubKey {
    musig_session_id: String,
    key_name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct MusigSessionId {
    musig_session_id: String
}

#[derive(Serialize, Deserialize, Debug)]
struct MusigSessionUpdateWithAggId {
    musig_session_id: String,
    musig_aggregation_id: String
}

#[derive(Serialize, Deserialize, Debug)]
struct OutputMusigSessionIdsList {
    musig_session_ids_list: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPairString {
    pub pub_key: String,
    pub sec_key: String
}

#[derive(Serialize, Deserialize, Debug)]
struct KeySessionName {
    session_: String,
    key_name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct OutputIsRegistered {
    is_registered:bool
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyNameId {
    key_name:String,
    key_id:String
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyId {
    key_id:String
}

pub fn register_user(cmd: String){
    let Ok(res) = serde_json::from_str::<InputRegistration>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as InputRegistration", cmd));
        return
    };
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    match klave::ledger::get_table("users_tab").get_string(&sender_id) {
        Ok(user) => {
            klave::notifier::send_string(&format!("ERROR: user already registered: '{}'", user));
            return
        }
        Err(_) => ()
    }
    match klave::ledger::get_table("users_tab").set_string(&sender_id, &res.user_name) {
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to register user: '{}'", e));
            return
        }
        Ok(()) => ()
    }
    match klave::ledger::get_table("users_id_tab").set_string(&res.user_name, &sender_id) {
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to register user x id: '{}'", e));
            return
        }
        Ok(()) => ()
    }
    //record all username in a list
    let mut username_list = vec![];
    match klave::ledger::get_table("users_id_tab").get_json::<Vec<String>>("ALL"){
        Ok(value) => {username_list = value},
        Err(_e) => ()
    }
    username_list.push(res.user_name.clone());
    match klave::ledger::get_table("users_id_tab").set_json::<Vec<String>>("ALL", &username_list) {
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to register global users list '{}'", e));
            return
        }
        Ok(()) => {
            klave::notifier::send_string(&res.user_name);
            return
        }
    }
}

pub fn load_current_user(_cmd: String){
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    let _ = klave::notifier::send_string(&sender_id);
    return
}
pub fn is_registered(_cmd: String){
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    let mut output_is_registered = OutputIsRegistered {
        is_registered: false
    };
    match klave::ledger::get_table("users_tab").get_string(&sender_id) {
        Ok(_) => {
            output_is_registered.is_registered = true;
            let _ = klave::notifier::send_json::<OutputIsRegistered>(&output_is_registered);
            return
        }
        Err(_) => {
            let _ = klave::notifier::send_json::<OutputIsRegistered>(&output_is_registered);
            return
        }
    }
}

pub fn get_all_users(_cmd: String){
    let mut username_list = vec![];
    match klave::ledger::get_table("users_id_tab").get_json::<Vec<String>>("ALL"){
        Ok(value) => {username_list = value},
        Err(_e) => ()
    }
    let _ = klave::notifier::send_json::<Vec<String>>(&username_list);
    return
}

pub fn create_key(cmd: String){
    //check input
    let Ok(res) = serde_json::from_str::<InputKeyCreation>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as InputKeyCreation", cmd));
        return
    };
    //get sender id
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    //retrieve list of user keys
    let mut user_key_ids_map: Map<String, Value> = match klave::ledger::get_table("user_key_ids_tab").get_json(&sender_id) {
        Ok(json) => json,
        Err(_) => Map::new()
    };
    if user_key_ids_map.contains_key(&res.key_name) {
        klave::notifier::send_string(&format!("ERROR: key with name '{}' already exists", res.key_name));
        return
    }
    //generate ecc key with klave
    // let Ok(key) = generate_ecc_key("secp256k1".to_string()) else {
    //     klave::notifier::send_string("ERROR: failed to generate key");
    //     return
    // };

    //save key with key id
    // let Ok(_) = save_key(&key, &key_id) else {
    //     klave::notifier::send_string("ERROR: failed to save key");
    //     return
    // };

    //generate new key
    let Ok(secret_key) = generate_secp256k1_secret_key() else {
        klave::notifier::send_string("ERROR: failed to generate secp256k1 key");
        return
    };
    let Ok(public_key) = generate_public_key_from_secret_key(secret_key.clone()) else {
        klave::notifier::send_string("ERROR: failed to generate public key");
        return
    };
    //generate random key id
    let Ok(key_id) = generate_rng_id() else {
        klave::notifier::send_string("ERROR: failed to generate key id");
        return
    };

    //save keys list
    user_key_ids_map.insert(res.key_name, Value::String(key_id.clone()));
    match klave::ledger::get_table("user_key_ids_tab").set_json(&sender_id, &user_key_ids_map) {
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to register user: '{}'", e));
            return
        }
        Ok(()) => {}
    }

    //save keys
    let key_pair = KeyPairString {
        pub_key: public_key,
        sec_key: secret_key
    };
    match klave::ledger::get_table("users_keys_tab").set_json(&key_id, &key_pair) {
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to save key: '{}'", e));
            return
        }
        Ok(()) => {}
    }
    klave::notifier::send_string("key created");
    return

}

pub fn load_keys(_cmd: String){
    //get sender id
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    let Ok(list_user_keys) = klave::ledger::get_table("user_key_ids_tab").get_json::<Map<String,Value>>(&sender_id) else {
        klave::notifier::send_string("ERROR: failed to get user keys");
        return
    };
    let _ = klave::notifier::send_json(&list_user_keys.keys().collect::<Vec<&String>>());
}

pub fn load_key_name_ids(_cmd: String){
    //get sender id
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    let Ok(map_user_keys) = klave::ledger::get_table("user_key_ids_tab").get_json::<Map<String,Value>>(&sender_id) else {
        klave::notifier::send_string("ERROR: failed to get user keys");
        return
    };
    let mut key_name_ids: Vec<KeyNameId> = vec![];
    for (key_name, key_id_value) in map_user_keys.iter() {
        let key_id = key_id_value.as_str().unwrap();
        let key_name2 = key_name.as_str();
        let key_name_id = KeyNameId{
            key_id: key_id.to_string(),
            key_name: key_name2.to_string()
        };
        key_name_ids.push(key_name_id);
    }
    let _ = klave::notifier::send_json::<Vec<KeyNameId>>(&key_name_ids);
}

pub fn load_key_pair(cmd: String){

    //check input
    let Ok(res) = serde_json::from_str::<KeyId>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as KeyId", cmd));
        return
    };
    //get key pair
    let Ok(key_pair_string) = klave::ledger::get_table("users_keys_tab").get_json::<KeyPairString>(&res.key_id) else {
        klave::notifier::send_string("ERROR: failed to get the key pair");
        return
    };
    //send key pair
    let _ = klave::notifier::send_json(&key_pair_string);
}

pub fn create_musig_session_definition(cmd: String){
    //check input
    let Ok(res) = serde_json::from_str::<InputMusigSessionInvitation>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as InputMusigSessionInvitation", cmd));
        return
    };
    //get sender id
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    //generate session id
    let Ok(musig_session_id) = crypto::random::get_random_bytes(32).map(|bytes| hex::encode(bytes)) else {
        klave::notifier::send_string("ERROR: failed to generate session id");
        return
    };
    //find the sender id list from the username list
    let mut sender_id_list: Vec<String> = vec![];
    for element in res.participants {
        let Ok(senderid) = klave::ledger::get_table("users_id_tab").get_string(&element) else {
            klave::notifier::send_string(&format!("ERROR: failed to find senderid for '{}'", element));
            return
        };
        sender_id_list.push(senderid);
    }
    //save musig session definition
    let session  = MusigSessionDefinition {
        session_name: res.session_name.clone(),
        session_id: musig_session_id.clone(),
        users_id: sender_id_list.clone(),
        owner_id: sender_id.clone(),
        public_keys: Map::new(),
        msg: res.message,
        signer_indexes: None,
        musig_aggregation_session: None
    };
    let Ok(_) = klave::ledger::get_table("musig_sessions_def_tab").set_json(&musig_session_id, &session) else {
        klave::notifier::send_string("ERROR: failed to save session");
        return
    };
    //retrieve list of user musig sessions
    for elem in sender_id_list {
        let mut list_musig_sessions: Vec<String> = match klave::ledger::get_table("musig_user_sessions_tab").get_json(&elem) {
            Ok(json) => json,
            Err(_) => Vec::new()
        };
        list_musig_sessions.push(musig_session_id.clone());
        match klave::ledger::get_table("musig_user_sessions_tab").set_json(&elem, &list_musig_sessions) {
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save user musig sessions: '{}'", e));
                return
            }
            Ok(()) => ()
        }
    }
    //return musig session id
    let output_musig_session_id = MusigSessionId {
        musig_session_id: musig_session_id
    };
    let _ = klave::notifier::send_json::<MusigSessionId>(&output_musig_session_id);
}

pub fn update_musig_user_public_key(cmd:String){
    let Ok(res) = serde_json::from_str::<InputMusigSessionUserPubKey>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as InputMusigSessionUserPubKey", cmd));
        return
    };
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };

    //TODO: test if this sender_id belongs to the musig session

    let Ok(mut session) = klave::ledger::get_table("musig_sessions_def_tab").get_json::<MusigSessionDefinition>(&res.musig_session_id) else {
        klave::notifier::send_string("ERROR: failed to load session");
        return
    };
    if session.public_keys.contains_key(&sender_id){
        klave::notifier::send_string("ERROR: failed to load session");
        return
    }

    //retrieve list of user keys
    let user_key_ids_map: Map<String, Value> = match klave::ledger::get_table("user_key_ids_tab").get_json(&sender_id) {
        Ok(json) => json,
        Err(_) => Map::new()
    };
    if !user_key_ids_map.contains_key(&res.key_name){
        klave::notifier::send_string("ERROR: failed to load key from key name");
        return
    }
    //retrieve key id
    let key_id;
    match user_key_ids_map.get(&res.key_name){
        Some(value) => {
            key_id = value.as_str().unwrap().to_string();
        },
        None => {
            klave::notifier::send_string("ERROR: failed to load key id");
            return
        }
    }
    //let key_id = user_key_ids_map.get(&res.key_name).unwrap().to_string();
    //retrieve key pair
    let key_pair_string: KeyPairString = match klave::ledger::get_table("users_keys_tab").get_json::<KeyPairString>(&key_id) {
        Ok(key) => key,
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to load key pair with error '{}'", e));
            return;
        }
    };

    session.public_keys.insert(sender_id.clone(), Value::String(key_pair_string.pub_key));
    //associate an index to each signer when everyone has provided its public key
    let nb_users = session.users_id.len();
    let mut map_sender_signer_index : HashMap<String, i8> = HashMap::new();

    if session.public_keys.len() == nb_users{
        let mut map_pk_sender : HashMap<PublicKey, String> = HashMap::new();
        let mut vec_pub_keys : Vec<PublicKey> = vec![];
        for (user_id, pub_key_str) in session.public_keys.iter() {
            // Handle the public key for each user
            let Ok(pub_key) = pub_key_str.as_str().unwrap().to_string().parse::<PublicKey>() else{
                klave::notifier::send_string("ERROR: failed to load public key");
                return
            };
            map_pk_sender.insert(pub_key, user_id.to_string());
            vec_pub_keys.push(pub_key);
        }
        vec_pub_keys.sort();

        let mut i = 0;
        for pk in vec_pub_keys {
            map_sender_signer_index.insert(map_pk_sender.get(&pk).unwrap().to_string(), i);
            i += 1;
        }
        session.signer_indexes = Some(map_sender_signer_index);
    }
    let Ok(_) = klave::ledger::get_table("musig_sessions_def_tab").set_json(&res.musig_session_id, &session) else {
        klave::notifier::send_string("ERROR: failed to save session");
        return
    };

    //TODO: clean this code, key_name shouldn't be needed
    let mut map_user_session_key = klave::ledger::get_table("musig_sessions_user_key_name").get_json::<Map<String,Value>>(&sender_id)
        .unwrap_or(Map::new());
    if !map_user_session_key.contains_key(&session.session_id)
    {
        map_user_session_key.insert(session.session_id.clone(), Value::String(res.key_name));
    }
    let Ok(_) = klave::ledger::get_table("musig_sessions_user_key_name").set_json(&sender_id, &map_user_session_key) else {
        klave::notifier::send_string("ERROR: failed to save session key name");
        return
    };

    //save musig session
    let Ok(_) = klave::ledger::get_table("musig_sessions_def_tab").set_json(&res.musig_session_id, &session) else {
        klave::notifier::send_string("ERROR: failed to save session");
        return
    };

    let _ = klave::notifier::send_json::<MusigSessionDefinition>(&session);
    return;
}

pub fn update_musig_aggregation_session(cmd: String){
    //check input
    let Ok(res) = serde_json::from_str::<MusigSessionUpdateWithAggId>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as MusigSessionUpdateWithAggId", cmd));
        return
    };
    //retrieve MusigSession
    let Ok(mut session) = klave::ledger::get_table("musig_sessions_def_tab").get_json::<MusigSessionDefinition>(&res.musig_session_id) else {
        klave::notifier::send_string("ERROR: failed to load session");
        return
    };
    //update musig aggregation id
    session.musig_aggregation_session = Some(res.musig_aggregation_id.clone());
    match klave::ledger::get_table("musig_sessions_def_tab").set_json(&res.musig_session_id, &session){
        Ok(_) => {
            let _ = klave::notifier::send_json::<MusigSessionDefinition>(&session);
        },
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to save musig session with error '{}'", e));
            return;
        }
    }
    return;
}

pub fn load_musig_session_ids(_cmd: String) {
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    let Ok(list_musig_sessions) = klave::ledger::get_table("musig_user_sessions_tab").get_json::<Vec<String>>(&sender_id) else {
        klave::notifier::send_string("ERROR: failed to get user musig sessions");
        return
    };
    let output_musig_session_list = OutputMusigSessionIdsList {
        musig_session_ids_list: list_musig_sessions
    };
    let _ = klave::notifier::send_json::<OutputMusigSessionIdsList>(&output_musig_session_list);
}

pub fn load_musig_session(cmd: String) {
    let Ok(res) = serde_json::from_str::<MusigSessionId>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as MusigSessionId", cmd));
        return
    };
    let Ok(sender_id) = klave::context::get("sender") else {
        klave::notifier::send_string("ERROR: failed to get the sender id");
        return
    };
    let Ok(session) = klave::ledger::get_table("musig_sessions_def_tab").get_json::<MusigSessionDefinition>(&res.musig_session_id) else {
        klave::notifier::send_string("ERROR: failed to load session");
        return
    };
    if !session.users_id.contains(&sender_id)
    {
        klave::notifier::send_string("ERROR: not allowed to load session");
        return
    }
    let _ = klave::notifier::send_json::<MusigSessionDefinition>(&session);
    return;
}