use tauri::command;
use serde::Serialize;
use theseus::{prelude::Credentials, auth, State};
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::{task::JoinHandle, sync::oneshot};

#[derive(Serialize)]
pub struct AuthUrl {
    pub url: String,
    pub id: String,
}

type FlowMap = Arc<RwLock<HashMap<Uuid, Arc<Mutex<JoinHandle<theseus::Result<Credentials>>>>>>>;

fn get_flows() -> &'static FlowMap {
    static FLOWS: FlowMap = Arc::new(RwLock::new(HashMap::new()));
    &FLOWS
}

#[command]
pub async fn authenticate_start() -> theseus::Result<AuthUrl> {
    let (tx, rx) = oneshot::channel::<url::Url>();
    let flow = Arc::new(Mutex::new(tokio::spawn(auth::authenticate(tx))));

    let url = rx.await?;//.map_err(|e| e.into())?;
    let url_str = url.to_string();

    println!("Adding new user account to Theseus");
    println!("A browser window will now open, follow the login flow there.");
    println!("URL {}", url_str);

    let id = Uuid::new_v4();
    get_flows().write().unwrap().insert(id, flow);

    Ok(AuthUrl { url: url_str, id: id.to_string() })
}

#[command]
pub async fn authenticate_complete(id: String) -> theseus::Result<Credentials> {
    let id = Uuid::parse_str(&id).map_err(|e| e.into())?;
    let flows = get_flows();
    let flow = flows.write().unwrap().remove(&id).ok_or_else(|| "Flow not found")?;

let flow = flow.lock().unwrap();
let credentials = flow.await.unwrap()?;
    State::sync().await?;
    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}
