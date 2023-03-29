#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::Path;

use theseus::prelude::*;
use tokio::sync::oneshot;

pub async fn authenticate_run() -> theseus::Result<Credentials> {
    println!("Adding new user account to Theseus");
    println!("A browser window will now open, follow the login flow there.");

    let (tx, rx) = oneshot::channel::<url::Url>();
    let flow = tokio::spawn(auth::authenticate(tx));

    let url = rx.await?;
    webbrowser::open(url.as_str())?;
    let credentials = flow.await.unwrap()?;
    State::sync().await?;
    println!("Logged in user {}.", credentials.username);
    Ok(credentials)
}

#[tokio::main]
async fn main() -> theseus::Result<()> {
    // Initialize state
    State::get().await?;

    // Clear profiles
    let h = profile::list().await?;
    for (path, _) in h.into_iter() {
        profile::remove(&path).await?;
    }

    // Craete vanilla minecraft instance
    // let profile = profile_create::profile_create_empty().await?;
    let path = Path::new("../.minecraft");
    profile::add_path(path).await?;

    // Dummy authentication
    let credentials = authenticate_run().await?;

    profile::run(&path.canonicalize()?, &credentials).await?;
    Ok(())
}
